<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { GRAMMAR_RULES, type GrammarCategory } from "$lib/grammar";
  import { playCardAudio } from "$lib/audio";

  interface Streak {
    streak_times: number;
    streak_koef: number;
  }

  interface Schedule {
    on_repeat: boolean;
    interval: number;
    next_recall: number;
    streak: Streak;
  }

  interface Card {
    id: number;
    kanji: string;
    kana: string;
    translation: string;
    example_japan?: string | null;
    example_trans?: string | null;
    recall_level?: string | null;
    life_time?: Schedule;
  }

  interface DeckStats {
    due_today: number;
    total_cards: number;
    repetition_count: number;
    learned_count: number;
  }

  interface DayActivity {
    day_name: string;
    date: string;
    done: boolean;
    is_today: boolean;
  }

  interface UserProgress {
    streak_days: number;
    today_completed: boolean;
    week_days: DayActivity[];
  }

  interface AppSettings {
    daily_quota: number;
    language: string;
  }

  type Screen = "dashboard" | "lesson" | "blitz" | "grammar" | "dictionary" | "settings";

  let currentScreen = $state<Screen>("dashboard");
  let currentLang = $state("Ru");

  let card = $state<Card | null>(null);
  let isRevealed = $state(false);
  let totalSessionCount = $state(10);
  let completedCount = $state(0);

  let stats = $state<DeckStats>({
    due_today: 0,
    total_cards: 0,
    repetition_count: 0,
    learned_count: 0,
  });

  const DEFAULT_WEEK_DAYS: DayActivity[] = [
    { day_name: "Пн", date: "", done: false, is_today: false },
    { day_name: "Вт", date: "", done: false, is_today: false },
    { day_name: "Ср", date: "", done: false, is_today: false },
    { day_name: "Чт", date: "", done: false, is_today: false },
    { day_name: "Пт", date: "", done: false, is_today: false },
    { day_name: "Сб", date: "", done: false, is_today: false },
    { day_name: "Вс", date: "", done: false, is_today: false },
  ];

  let userProgress = $state<UserProgress>({
    streak_days: 0,
    today_completed: false,
    week_days: DEFAULT_WEEK_DAYS,
  });

  let appSettings = $state<AppSettings>({
    daily_quota: 10,
    language: "Ru",
  });

  const todayFormatted = new Intl.DateTimeFormat('ru-RU', { day: 'numeric', month: 'long' }).format(new Date());

  // Словарь (5.0 単語帳)
  let allCards = $state<Card[]>([]);
  let dictSearch = $state("");
  type DictFilter = "all" | "learned" | "review" | "new";
  let dictFilter = $state<DictFilter>("all");
  type DictSort = "id" | "kana" | "interval";
  let dictSort = $state<DictSort>("id");

  let filteredCards = $derived.by(() => {
    let result = allCards;

    // 1. Фильтр по статусу изучения
    if (dictFilter === "learned") {
      result = result.filter((c) => (c.life_time?.streak?.streak_times ?? 0) > 0);
    } else if (dictFilter === "review") {
      result = result.filter((c) => c.life_time?.on_repeat);
    } else if (dictFilter === "new") {
      result = result.filter(
        (c) => (c.life_time?.streak?.streak_times ?? 0) === 0 && !c.life_time?.on_repeat
      );
    }

    // 2. Поиск по подстроке
    const q = dictSearch.trim().toLowerCase();
    if (q) {
      result = result.filter((c) =>
        c.kanji.toLowerCase().includes(q) ||
        c.kana.toLowerCase().includes(q) ||
        c.translation.toLowerCase().includes(q) ||
        (c.example_japan && c.example_japan.toLowerCase().includes(q)) ||
        (c.example_trans && c.example_trans.toLowerCase().includes(q))
      );
    }

    // 3. Сортировка
    return [...result].sort((a, b) => {
      if (dictSort === "kana") {
        return a.kana.localeCompare(b.kana, "ja");
      } else if (dictSort === "interval") {
        return (b.life_time?.interval ?? 0) - (a.life_time?.interval ?? 0);
      } else {
        return a.id - b.id;
      }
    });
  });

  async function loadAllCards() {
    try {
      allCards = await invoke<Card[]>("get_all_cards");
    } catch (e) {
      console.error("Failed to load all cards", e);
    }
  }

  // Грамматика
  let grammarSearch = $state("");
  let grammarCategory = $state<GrammarCategory>("all");

  let filteredGrammar = $derived(
    GRAMMAR_RULES.filter((rule) => {
      const matchesCategory = grammarCategory === "all" || rule.category === grammarCategory;
      if (!matchesCategory) return false;
      if (!grammarSearch.trim()) return true;
      const q = grammarSearch.toLowerCase().trim();
      return (
        rule.title.toLowerCase().includes(q) ||
        rule.formula.toLowerCase().includes(q) ||
        rule.desc.toLowerCase().includes(q) ||
        rule.examples.some((ex) => ex.jp.includes(q) || ex.ru.toLowerCase().includes(q))
      );
    })
  );

  // Спринт (60 сек)
  let blitzTimer = $state(60);
  let blitzScore = $state(0);
  let blitzActive = $state(false);
  let blitzInterval: any = null;
  let blitzCard = $state<Card | null>(null);

  async function loadCard() {
    card = await invoke<Card | null>("get_current_card");
    isRevealed = false;
  }

  async function loadStats() {
    try {
      stats = await invoke<DeckStats>("get_deck_stats");
      if (stats.due_today > 0 && completedCount === 0) {
        totalSessionCount = Math.min(stats.due_today, 10);
      }
    } catch (e) {
      console.error("Failed to load deck stats", e);
    }
  }

  async function loadUserProgress() {
    try {
      const prog = await invoke<UserProgress>("get_user_progress");
      if (prog && prog.week_days && prog.week_days.length === 7) {
        userProgress = prog;
      } else if (prog) {
        userProgress = { ...prog, week_days: DEFAULT_WEEK_DAYS };
      }
    } catch (e) {
      console.error("Failed to load user progress", e);
    }
  }

  async function loadAppSettings() {
    try {
      appSettings = await invoke<AppSettings>("get_app_settings");
      currentLang = appSettings.language;
    } catch (e) {
      console.error("Failed to load app settings", e);
    }
  }

  async function saveSettings(quota: number, lang: string) {
    try {
      await invoke("save_app_settings", { dailyQuota: quota, language: lang });
      appSettings.daily_quota = quota;
      appSettings.language = lang;
      currentLang = lang;
    } catch (e) {
      console.error("Failed to save app settings", e);
    }
  }

  async function loadNextBlitzCard() {
    blitzCard = await invoke<Card | null>("get_random_card");
  }

  async function handleReview(button: "Instantly" | "Difficult" | "Repeat") {
    if (button !== "Repeat") {
      completedCount += 1;
    }
    card = await invoke<Card | null>("review", { button });
    isRevealed = false;
    if (!card) {
      await loadStats();
      await loadAllCards();
      await loadUserProgress();
    }
  }

  function flipCard() {
    if (!isRevealed) {
      isRevealed = true;
      if (card) {
        playCardAudio(card.id);
      }
    }
  }

  function navigateTo(screen: Screen) {
    currentScreen = screen;
    if (screen === "lesson") {
      loadCard();
    } else if (screen === "dashboard") {
      loadStats();
      loadUserProgress();
    } else if (screen === "dictionary") {
      loadAllCards();
    } else if (screen === "settings") {
      loadAppSettings();
    }
    if (screen !== "blitz") {
      resetBlitz();
    }
  }

  async function startBlitz() {
    blitzActive = true;
    blitzTimer = 60;
    blitzScore = 0;
    await loadNextBlitzCard();
    clearInterval(blitzInterval);
    blitzInterval = setInterval(() => {
      if (blitzTimer > 0) {
        blitzTimer -= 1;
      } else {
        stopBlitz();
      }
    }, 1000);
  }

  function stopBlitz() {
    blitzActive = false;
    clearInterval(blitzInterval);
  }

  function resetBlitz() {
    blitzActive = false;
    clearInterval(blitzInterval);
    blitzTimer = 60;
    blitzScore = 0;
    blitzCard = null;
  }

  function blitzKnown() {
    if (!blitzActive) return;
    blitzScore += 1;
    loadNextBlitzCard();
  }

  function blitzSkip() {
    if (!blitzActive) return;
    loadNextBlitzCard();
  }

  // Обработка горячих клавиш (Keyboard-First UX)
  function handleKeydown(event: KeyboardEvent) {
    // Навигация: Esc всегда возвращает на дашборд
    if (event.key === "Escape") {
      if (currentScreen !== "dashboard") {
        navigateTo("dashboard");
      }
      return;
    }

    // Горячие клавиши на главном экране: Enter начинает урок
    if (currentScreen === "dashboard") {
      if (event.key === "Enter") {
        navigateTo("lesson");
      }
      return;
    }

    // Горячие клавиши на экране урока
    if (currentScreen === "lesson") {
      if (!card) {
        // Экран завершения: Enter возвращает на главную
        if (event.key === "Enter") {
          navigateTo("dashboard");
        }
        return;
      }

      // Горячая клавиша R: повтор эталонного аудио
      if (event.key === "r" || event.key === "R" || event.key === "к" || event.key === "К") {
        event.preventDefault();
        playCardAudio(card.id);
        return;
      }

      if (!isRevealed) {
        // Если карта закрыта: Пробел или Enter открывают её
        if (event.key === " " || event.key === "Enter") {
          event.preventDefault();
          flipCard();
        }
      } else {
        // Если карта раскрыта: 1, 2, 3 отправляют оценку
        if (event.key === "1") {
          handleReview("Repeat");
        } else if (event.key === "2") {
          handleReview("Difficult");
        } else if (event.key === "3") {
          handleReview("Instantly");
        }
      }
      return;
    }

    // Горячие клавиши на экране спринта
    if (currentScreen === "blitz") {
      if (!blitzActive) {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          startBlitz();
        }
        return;
      }

      if (event.key === "1" || event.key === "Enter" || event.key === " ") {
        event.preventDefault();
        blitzKnown();
      } else if (event.key === "2" || event.key === "ArrowRight") {
        event.preventDefault();
        blitzSkip();
      }
      return;
    }
  }

  onMount(() => {
    loadStats();
    loadCard();
    loadAllCards();
    loadUserProgress();
    loadAppSettings();
    window.addEventListener("keydown", handleKeydown);
    return () => {
      window.removeEventListener("keydown", handleKeydown);
      resetBlitz();
    };
  });
</script>

<main class="app-viewport">
  <!-- ВСПЛЫВАЮЩИЙ САЙДБАР (Стиль японского книжного каталога) -->
  <aside class="sidebar-hover-zone">
    <div class="sidebar-panel">
      <div class="sidebar-brand">
        <span class="brand-hanko">●</span>
        <div class="brand-text">
          <span class="brand-jp">日本語学習</span>
          <span class="brand-sub">Kotoba App</span>
        </div>
      </div>

      <nav class="sidebar-nav">
        <button 
          class="nav-item" 
          class:active={currentScreen === "dashboard"} 
          onclick={() => navigateTo("dashboard")}
        >
          <span class="nav-idx">1.0</span>
          <span class="nav-label">ホーム (Главная)</span>
        </button>

        <button 
          class="nav-item" 
          class:active={currentScreen === "lesson"} 
          onclick={() => navigateTo("lesson")}
        >
          <span class="nav-idx">2.0</span>
          <span class="nav-label">レッスン (Ежедневный урок)</span>
        </button>

        <button 
          class="nav-item" 
          class:active={currentScreen === "blitz"} 
          onclick={() => navigateTo("blitz")}
        >
          <span class="nav-idx">3.0</span>
          <span class="nav-label">スプリント (Спринт 60с)</span>
        </button>

        <button 
          class="nav-item" 
          class:active={currentScreen === "grammar"} 
          onclick={() => navigateTo("grammar")}
        >
          <span class="nav-idx">4.0</span>
          <span class="nav-label">文法 (Грамматика)</span>
        </button>

        <button 
          class="nav-item" 
          class:active={currentScreen === "dictionary"} 
          onclick={() => navigateTo("dictionary")}
        >
          <span class="nav-idx">5.0</span>
          <span class="nav-label">単語帳 (Мой словарь)</span>
        </button>

        <button 
          class="nav-item" 
          class:active={currentScreen === "settings"} 
          onclick={() => navigateTo("settings")}
        >
          <span class="nav-idx">6.0</span>
          <span class="nav-label">設定 (Настройки)</span>
        </button>
      </nav>
    </div>
  </aside>

  <!-- ВЕРХНЯЯ СТРОГАЯ ЛИНИЯ НАВИГАЦИИ -->
  <header class="top-nav">
    <div class="nav-left">
      <span class="hanko-circle"></span>
      <span class="nav-title">言葉 KOTOBA</span>
      <span class="nav-level-tag">JLPT N5</span>
    </div>

    {#if currentScreen !== "dashboard"}
      <button class="nav-back-link" onclick={() => navigateTo("dashboard")}>
        ← На главную
      </button>
    {:else}
      <div class="streak-badge" title="Серия занятий без пропусков">
        <span class="streak-fire">🔥</span>
        <span class="streak-num">{userProgress.streak_days}</span>
        <span class="streak-label">{userProgress.streak_days === 1 ? 'день' : (userProgress.streak_days >= 2 && userProgress.streak_days <= 4) ? 'дня' : 'дней'} подряд</span>
      </div>
    {/if}
  </header>

  <!-- ================== ЭКРАН 1: ГЛАВНЫЙ (ВАРИАНТ А: EDITORIAL BOOKSPREAD) ================== -->
  {#if currentScreen === "dashboard"}
    <div class="content-view dashboard-layout-spread">
      <!-- Заголовок дня -->
      <div class="spread-header">
        <div class="date-meta">{todayFormatted} • {userProgress.streak_days > 0 ? `${userProgress.streak_days} 日目` : '1 日目'}</div>
        <h1 class="page-title">Сегодняшний разворот</h1>
        <p class="spread-subtitle">
          {#if userProgress.today_completed}
            Все запланированные карточки на сегодня освоены!
          {:else if stats.due_today > 0}
            {stats.due_today} {stats.due_today === 1 ? 'выражение' : stats.due_today < 5 ? 'выражения' : 'выражений'} к освоению. Оценочное время: ~{Math.max(1, Math.ceil(stats.due_today * 0.5))} мин.
          {:else}
            Все запланированные карточки на сегодня освоены!
          {/if}
        </p>
      </div>

      <!-- Главный книжный разворот: Сетка слов слева + Панель действия справа -->
      <div class="bookspread-board">
        <!-- Левая часть: Реальная сетка иероглифов сегодняшней сессии -->
        <div class="spread-words-col">
          <div class="col-meta-header">
            <span>ПРЕВЬЮ СЛОВАРЯ ({stats.total_cards || 60})</span>
            <span class="col-sub">JLPT N5</span>
          </div>

          <div class="kanji-matrix-grid">
            <div class="matrix-cell">
              <span class="m-kana">たべる</span>
              <span class="m-kanji">食べる</span>
              <span class="m-ru">есть</span>
            </div>
            <div class="matrix-cell">
              <span class="m-kana">のむ</span>
              <span class="m-kanji">飲む</span>
              <span class="m-ru">пить</span>
            </div>
            <div class="matrix-cell">
              <span class="m-kana">ほん</span>
              <span class="m-kanji">本</span>
              <span class="m-ru">книга</span>
            </div>
            <div class="matrix-cell">
              <span class="m-kana">ねこ</span>
              <span class="m-kanji">猫</span>
              <span class="m-ru">кошка</span>
            </div>
            <div class="matrix-cell">
              <span class="m-kana">いぬ</span>
              <span class="m-kanji">犬</span>
              <span class="m-ru">собака</span>
            </div>
            <div class="matrix-cell">
              <span class="m-kana">みず</span>
              <span class="m-kanji">水</span>
              <span class="m-ru">вода</span>
            </div>
            <div class="matrix-cell">
              <span class="m-kana">いく</span>
              <span class="m-kanji">行く</span>
              <span class="m-ru">идти</span>
            </div>
            <div class="matrix-cell">
              <span class="m-kana">みる</span>
              <span class="m-kanji">見る</span>
              <span class="m-ru">смотреть</span>
            </div>
            <div class="matrix-cell">
              <span class="m-kana">ともだち</span>
              <span class="m-kanji">友達</span>
              <span class="m-ru">друг</span>
            </div>
            <div class="matrix-cell">
              <span class="m-kana">にほんご</span>
              <span class="m-kanji">日本語</span>
              <span class="m-ru">японский</span>
            </div>
          </div>
        </div>

        <!-- Правая часть: Сводка и кнопка старта -->
        <div class="spread-action-col">
          <div class="session-brief">
            <div class="brief-item">
              <span class="brief-label">К повторению</span>
              <strong class="brief-val">{stats.due_today}</strong>
            </div>
            <div class="brief-item">
              <span class="brief-label">На повторении</span>
              <strong class="brief-val">{stats.repetition_count}</strong>
            </div>
            <div class="brief-item-mastery">
              <div class="mastery-label-row">
                <span class="brief-label">Освоение колоды N5</span>
                <strong class="brief-val">
                  {stats.total_cards > 0 ? Math.round((stats.learned_count / stats.total_cards) * 100) : 0}%
                </strong>
              </div>
              <div class="mini-progress-track">
                <div 
                  class="mini-progress-fill" 
                  style="width: {stats.total_cards > 0 ? Math.min(100, (stats.learned_count / stats.total_cards) * 100) : 0}%"
                ></div>
              </div>
              <span class="mastery-subtext">{stats.learned_count} из {stats.total_cards} базовых слов</span>
            </div>
          </div>

          <div class="action-dock-box">
            <button class="action-btn-primary spread-start-btn" onclick={() => navigateTo("lesson")}>
              {#if userProgress.today_completed}
                Урок пройден ✓
              {:else if stats.due_today > 0}
                Открыть урок →
              {:else}
                Повторить урок →
              {/if}
            </button>
            <span class="hotkey-hint">Клавиша: Enter</span>
          </div>
        </div>
      </div>

      <!-- Нижний ритм недели -->
      <div class="rhythm-strip">
        <span class="rhythm-title">НЕДЕЛЬНЫЙ РИТМ</span>
        <div class="rhythm-divider-vertical"></div>
        <div class="rhythm-dots">
          {#each userProgress.week_days as day}
            <span class="r-day" class:done={day.done} class:today={day.is_today} class:wait={!day.done && !day.is_today}>
              <span class="r-mark">{day.done ? '●' : '○'}</span> {day.day_name}
            </span>
          {/each}
        </div>
      </div>
    </div>

  <!-- ================== ЭКРАН 2: УРОК (ДВЕ БУМАЖНЫЕ КАРТЫ С ТУШЬЮ) ================== -->
  {:else if currentScreen === "lesson"}
    <div class="content-view lesson-layout">
      {#if card}
        <div class="lesson-meta-bar">
          <div class="session-progress-bar">
            <div class="progress-fill" style="width: {(completedCount / totalSessionCount) * 100}%"></div>
          </div>
          <div class="progress-label-row">
            <span>Прогресс: <strong>{completedCount}</strong> из <strong>{totalSessionCount}</strong></span>
            <span class="esc-hint">Esc: вернуться назад</span>
          </div>
        </div>

        <div class="cards-stage" class:revealed={isRevealed}>
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div class="paper-card card-japanese" onclick={flipCard}>
            <div class="card-top-row">
              <span class="card-tag">単語 • Слово</span>
              <button 
                type="button" 
                class="card-audio-btn" 
                onclick={(e) => { e.stopPropagation(); if (card) playCardAudio(card.id); }}
                title="Прослушать произношение [R]"
              >
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"></polygon>
                  <path d="M15.54 8.46a5 5 0 0 1 0 7.07"></path>
                  <path d="M19.07 4.93a10 10 0 0 1 0 14.14"></path>
                </svg>
                <span class="audio-btn-kbd">R</span>
              </button>
            </div>
            <div class="furigana-top">{card.kana}</div>
            <h1 class="kanji-display">{card.kanji}</h1>
            {#if !isRevealed}
              <div class="reveal-hint">
                <span class="kbd-pill">Пробел</span> или клик для перевода
              </div>
            {/if}
          </div>

          {#if isRevealed}
            <div class="paper-card card-translation">
              <div class="card-tag">意味と文脈 • Значение и контекст</div>
              
              <div class="translation-head">
                <h2 class="translation-title">{card.translation}</h2>
              </div>

              {#if card.example_japan}
                <div class="example-box">
                  <div class="example-label">例文 (Пример):</div>
                  <div class="example-jp">{card.example_japan}</div>
                  {#if card.example_trans}
                    <div class="example-target">{card.example_trans}</div>
                  {/if}
                </div>
              {/if}
            </div>
          {/if}
        </div>

        <!-- Кнопки оценки в виде строгой бумажной панели -->
        <div class="review-dock" class:visible={isRevealed}>
          <button class="review-btn btn-again" onclick={() => handleReview("Repeat")}>
            <span class="btn-kbd">1</span>
            <span>Ещё раз (もう一度)</span>
          </button>
          <button class="review-btn btn-hard" onclick={() => handleReview("Difficult")}>
            <span class="btn-kbd">2</span>
            <span>Трудно (難しい)</span>
          </button>
          <button class="review-btn btn-good" onclick={() => handleReview("Instantly")}>
            <span class="btn-kbd">3</span>
            <span>Хорошо (覚えた)</span>
          </button>
        </div>
      {:else}
        <div class="paper-completion-card">
          <div class="seal-mark">完</div>
          <h2 class="completion-title">Сессия успешно завершена</h2>
          <p class="completion-desc">
            Все запланированные карточки отработаны. Алгоритм пересчитал интервалы и распределил повторения по дням.
          </p>
          <div class="completion-stats">
            <div class="c-stat">
              <span class="c-val">{completedCount}</span>
              <span class="c-label">слов пройдено</span>
            </div>
            <div class="c-stat">
              <span class="c-val">{stats.learned_count}</span>
              <span class="c-label">всего освоено</span>
            </div>
          </div>
          <button class="action-btn-primary completion-btn" onclick={() => navigateTo("dashboard")}>
            Вернуться на разворот <span class="btn-sub-kbd">[Enter]</span>
          </button>
        </div>
      {/if}
    </div>

  <!-- ================== ЭКРАН 3: СПРИНТ ================== -->
  {:else if currentScreen === "blitz"}
    <div class="content-view blitz-layout">
      <div class="section-badge">3.0 速度練習 — Проверка на скорость</div>
      
      <div class="paper-blitz-box">
        <div class="blitz-meta-line">
          <span class="timer-text">Осталось: <strong>{blitzTimer}с</strong></span>
          <span class="score-text">Очки: <strong>{blitzScore}</strong></span>
        </div>

        {#if !blitzActive && blitzTimer === 60}
          <div class="blitz-intro">
            <h2>60 секунд — максимум слов</h2>
            <p>Тренировка быстрого чтения без пауз и долгих раздумий.</p>
            <button class="action-btn-primary" onclick={startBlitz}>
              Начать спринт <span class="btn-sub-kbd">[Enter]</span>
            </button>
          </div>
        {:else if !blitzActive && blitzTimer === 0}
          <div class="blitz-intro">
            <h2>Время истекло</h2>
            <p>Правильных ответов: <strong>{blitzScore}</strong></p>
            <button class="action-btn-primary" onclick={startBlitz}>
              Повторить <span class="btn-sub-kbd">[Enter]</span>
            </button>
          </div>
        {:else if blitzActive && blitzCard}
          <div class="blitz-word-stage">
            <div class="blitz-kanji">{blitzCard.kanji}</div>
            <div class="blitz-kana">{blitzCard.kana}</div>

            <div class="blitz-choice-row">
              <button class="action-btn-primary" onclick={blitzKnown}>
                Знаю <span class="btn-sub-kbd">[1]</span>
              </button>
              <button class="action-btn-outline" onclick={blitzSkip}>
                Пропустить <span class="btn-sub-kbd">[2]</span>
              </button>
            </div>
          </div>
        {:else if blitzActive}
          <div class="blitz-intro">
            <p>Загрузка карточек...</p>
          </div>
        {:else}
          <div class="blitz-intro">
            <h2>60 секунд — максимум слов</h2>
            <p>Тренировка быстрого чтения без пауз и долгих раздумий.</p>
            <button class="action-btn-primary" onclick={startBlitz}>
              Начать спринт <span class="btn-sub-kbd">[Enter]</span>
            </button>
          </div>
        {/if}
      </div>
    </div>

  <!-- ================== ЭКРАН 4: ГРАММАТИКА ================== -->
  {:else if currentScreen === "grammar"}
    <div class="content-view grammar-layout">
      <div class="section-badge">4.0 基本文法 — Справочник грамматики (JLPT N5)</div>
      
      <div class="grammar-header-row">
        <div>
          <h1 class="page-title">Справочник грамматики</h1>
          <p class="grammar-subtitle">
            16 фундаментальных грамматических правил и частиц для сдачи норматива N5
          </p>
        </div>
      </div>

      <!-- Каталожные вкладки (目次 / Реестр разделов) -->
      <div class="grammar-toolbar">
        <nav class="grammar-index-nav">
          <button 
            class="index-tab" 
            class:active={grammarCategory === "all"} 
            onclick={() => grammarCategory = "all"}
          >
            <span class="tab-jp">全項目</span>
            <span class="tab-ru">Все правила</span>
            <span class="tab-count">[{GRAMMAR_RULES.length}]</span>
          </button>
          <button 
            class="index-tab" 
            class:active={grammarCategory === "particles"} 
            onclick={() => grammarCategory = "particles"}
          >
            <span class="tab-jp">助詞</span>
            <span class="tab-ru">Частицы</span>
            <span class="tab-count">[{GRAMMAR_RULES.filter(r => r.category === 'particles').length}]</span>
          </button>
          <button 
            class="index-tab" 
            class:active={grammarCategory === "verbs"} 
            onclick={() => grammarCategory = "verbs"}
          >
            <span class="tab-jp">動詞</span>
            <span class="tab-ru">Глаголы</span>
            <span class="tab-count">[{GRAMMAR_RULES.filter(r => r.category === 'verbs').length}]</span>
          </button>
          <button 
            class="index-tab" 
            class:active={grammarCategory === "patterns"} 
            onclick={() => grammarCategory = "patterns"}
          >
            <span class="tab-jp">文型</span>
            <span class="tab-ru">Конструкции</span>
            <span class="tab-count">[{GRAMMAR_RULES.filter(r => r.category === 'patterns').length}]</span>
          </button>
        </nav>

        <div class="grammar-search-wrap">
          <span class="search-tag">検索 • Поиск</span>
          <div class="grammar-search-box">
            <input 
              type="text" 
              bind:value={grammarSearch} 
              placeholder="Введите частицу, формулу или перевод..." 
              class="grammar-search-input"
            />
            {#if grammarSearch}
              <button class="search-clear-btn" onclick={() => grammarSearch = ""}>✕</button>
            {/if}
          </div>
        </div>
      </div>

      <!-- Список статей справочника -->
      <div class="grammar-table-card">
        {#if filteredGrammar.length === 0}
          <div class="grammar-not-found">
            <p>Ничего не найдено по запросу «{grammarSearch}»</p>
            <button class="action-btn-outline" onclick={() => { grammarSearch = ''; grammarCategory = 'all'; }}>
              Сбросить фильтры
            </button>
          </div>
        {:else}
          {#each filteredGrammar as rule (rule.id)}
            <article class="grammar-entry">
              <div class="entry-index">{rule.id}</div>
              <div class="entry-content">
                <div class="entry-top-meta">
                  <span class="entry-cat-badge">{rule.categoryLabel}</span>
                </div>
                <h3>{rule.title}</h3>
                <p class="formula-line"><code>{rule.formula}</code></p>
                <p class="entry-desc">{rule.desc}</p>
                
                <div class="examples-list">
                  {#each rule.examples as ex}
                    <div class="entry-example">
                      <div class="example-jp-line">
                        <span class="ex-jp">{ex.jp}</span>
                        {#if ex.kana}
                          <span class="ex-kana">({ex.kana})</span>
                        {/if}
                      </div>
                      <div class="ex-ru">{ex.ru}</div>
                    </div>
                  {/each}
                </div>
              </div>
            </article>
          {/each}
        {/if}
      </div>
    </div>

  <!-- ================== ЭКРАН 5: МОЙ СЛОВАРЬ ================== -->
  {:else if currentScreen === "dictionary"}
    <div class="content-view dict-layout">
      <div class="section-badge">5.0 私の単語帳 — Каталог лексики и личный словарь (JLPT N5)</div>
      
      <div class="dict-header-row">
        <div>
          <h1 class="page-title">Словарь выражений</h1>
          <p class="dict-subtitle">
            Все {allCards.length} слов из базы данных с персональными интервалами повторения
          </p>
        </div>
      </div>

      <!-- Каталожные вкладки и строка поиска -->
      <div class="dict-toolbar">
        <nav class="dict-index-nav">
          <button 
            class="index-tab" 
            class:active={dictFilter === "all"} 
            onclick={() => dictFilter = "all"}
          >
            <span class="tab-jp">全単語</span>
            <span class="tab-ru">Все слова</span>
            <span class="tab-count">[{allCards.length}]</span>
          </button>
          <button 
            class="index-tab" 
            class:active={dictFilter === "learned"} 
            onclick={() => dictFilter = "learned"}
          >
            <span class="tab-jp">習得済</span>
            <span class="tab-ru">Освоенные</span>
            <span class="tab-count">[{allCards.filter(c => (c.life_time?.streak?.streak_times ?? 0) > 0).length}]</span>
          </button>
          <button 
            class="index-tab" 
            class:active={dictFilter === "review"} 
            onclick={() => dictFilter = "review"}
          >
            <span class="tab-jp">復習中</span>
            <span class="tab-ru">На повторении</span>
            <span class="tab-count">[{allCards.filter(c => c.life_time?.on_repeat).length}]</span>
          </button>
          <button 
            class="index-tab" 
            class:active={dictFilter === "new"} 
            onclick={() => dictFilter = "new"}
          >
            <span class="tab-jp">未習</span>
            <span class="tab-ru">Новые</span>
            <span class="tab-count">[{allCards.filter(c => (c.life_time?.streak?.streak_times ?? 0) === 0 && !c.life_time?.on_repeat).length}]</span>
          </button>
        </nav>

        <div class="dict-controls-row">
          <div class="dict-search-wrap">
            <span class="search-tag">単語検索</span>
            <div class="grammar-search-box">
              <input 
                type="text" 
                bind:value={dictSearch} 
                placeholder="Поиск по иероглифу, кане или значению..." 
                class="grammar-search-input"
              />
              {#if dictSearch}
                <button class="search-clear-btn" onclick={() => dictSearch = ""}>✕</button>
              {/if}
            </div>
          </div>

          <div class="dict-sort-wrap">
            <span class="sort-label">Порядок:</span>
            <select bind:value={dictSort} class="dict-sort-select">
              <option value="id">По номеру в базе (1..{allCards.length})</option>
              <option value="kana">По азбуке каны (五十音順)</option>
              <option value="interval">По интервалу SRS (длинные сначала)</option>
            </select>
          </div>
        </div>
      </div>

      <!-- Реестр слов -->
      <div class="dict-table-card">
        {#if filteredCards.length === 0}
          <div class="grammar-not-found">
            <p>В словаре не найдено слов по заданным фильтрам</p>
            <button class="action-btn-outline" onclick={() => { dictSearch = ''; dictFilter = 'all'; }}>
              Сбросить фильтры
            </button>
          </div>
        {:else}
          {#each filteredCards as word (word.id)}
            <article class="dict-entry">
              <div class="dict-index">{word.id < 10 ? `0${word.id}` : word.id}</div>
              
              <div class="dict-main-col">
                <div class="dict-kana-line">
                  <span>{word.kana}</span>
                  <button 
                    type="button" 
                    class="dict-mini-audio-btn" 
                    onclick={(e) => { e.stopPropagation(); playCardAudio(word.id); }}
                    title="Прослушать {word.kana}"
                  >
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                      <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"></polygon>
                      <path d="M15.54 8.46a5 5 0 0 1 0 7.07"></path>
                    </svg>
                  </button>
                </div>
                <div class="dict-kanji-line">{word.kanji}</div>
              </div>

              <div class="dict-trans-col">
                <div class="dict-translation">{word.translation}</div>
                {#if word.example_japan}
                  <div class="dict-example">
                    <span class="ex-label">例:</span>
                    <span class="ex-jp">{word.example_japan}</span>
                    {#if word.example_trans}
                      <span class="ex-sep">•</span>
                      <span class="ex-ru">{word.example_trans}</span>
                    {/if}
                  </div>
                {/if}
              </div>

              <div class="dict-meta-col">
                {#if word.life_time?.on_repeat}
                  <span class="srs-badge badge-repeat">На повторении</span>
                {:else if (word.life_time?.streak?.streak_times ?? 0) > 0}
                  <span class="srs-badge badge-learned">
                    <span class="badge-fire">🔥</span>
                    <span>{word.life_time?.interval}д интервал</span>
                  </span>
                {:else}
                  <span class="srs-badge badge-new">Новое</span>
                {/if}
              </div>
            </article>
          {/each}
        {/if}
      </div>
    </div>

  <!-- ================== ЭКРАН 6: НАСТРОЙКИ ================== -->
  {:else if currentScreen === "settings"}
    <div class="content-view settings-layout">
      <div class="section-badge">6.0 アプリ設定 — Конфигурация</div>
      <h1 class="page-title">Настройки</h1>

      <div class="settings-table">
        <div class="settings-item">
          <div>
            <strong>Целевой язык перевода</strong>
            <p>Язык, на котором отображаются значения и примеры предложений</p>
          </div>
          <select 
            bind:value={appSettings.language} 
            onchange={() => saveSettings(appSettings.daily_quota, appSettings.language)}
            class="paper-select"
          >
            <option value="Ru">Русский (Ru)</option>
            <option value="En">English (En)</option>
            <option value="Es">Español (Es)</option>
            <option value="De">Deutsch (De)</option>
          </select>
        </div>

        <div class="settings-item">
          <div>
            <strong>Размер ежедневной колоды</strong>
            <p>Количество новых слов для каждой сессии</p>
          </div>
          <input 
            type="number" 
            min="5" 
            max="50" 
            bind:value={appSettings.daily_quota} 
            onchange={() => saveSettings(Number(appSettings.daily_quota) || 10, appSettings.language)}
            class="paper-input" 
          />
        </div>

        <div class="settings-item">
          <div>
            <strong>Внешние словари</strong>
            <p>Подключение файлов JSON со сторонними списками слов</p>
          </div>
          <button class="action-btn-outline" onclick={() => alert("Импорт файлов будет доступен в следующем модуле")}>
            Выбрать .json файл
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Иконка настроек в нижнем правом углу -->
  <button 
    class="bottom-settings-btn" 
    class:active={currentScreen === "settings"}
    onclick={() => navigateTo(currentScreen === "settings" ? "dashboard" : "settings")}
    title="Настройки (設定)"
  >
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
      <circle cx="12" cy="12" r="3"></circle>
      <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
    </svg>
  </button>
</main>

<style>
  /* ================== ЯПОНСКАЯ ПОЛИГРАФИЯ (MUJI / WASHI) ================== */
  :global(:root) {
    --paper-bg: #F7F5F0;             /* Молочно-кремовая бумага */
    --paper-card: #FFFFFF;           /* Чистый бумажный лист */
    --ink-black: #1A1816;            /* Угольные чернила */
    --ink-secondary: #57534E;        /* Вторичный серый текст */
    --ink-faint: #8C867E;            /* Бледно-серые подписи */
    --border-line: #E7E3DA;          /* Строгая тонкая линия 1px */
    --border-line-dark: #D4CEBF;     /* Усиленная граница */
    --hanko-red: #C83226;            /* Традиционная японская киноварь (朱色) */
    --hanko-faint: rgba(200, 50, 38, 0.08);
  }

  :global(body) {
    margin: 0;
    padding: 0;
    font-family: -apple-system, BlinkMacSystemFont, "Hiragino Mincho ProN", "Yu Mincho", "Noto Serif JP", serif;
    background-color: var(--paper-bg);
    color: var(--ink-black);
    height: 100vh;
    display: flex;
    overflow: hidden;
    user-select: none;
    -webkit-font-smoothing: antialiased;
  }

  .app-viewport {
    flex: 1;
    display: flex;
    flex-direction: column;
    height: 100vh;
    padding: 0 48px 48px 48px;
    box-sizing: border-box;
    position: relative;
    overflow-y: auto;
  }

  /* ================== ВЕРХНЯЯ СТРОГАЯ НАВИГАЦИЯ ================== */
  .top-nav {
    width: 100%;
    max-width: 860px;
    margin: 0 auto 36px auto;
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 20px 0;
    border-bottom: 1px solid var(--border-line);
    box-sizing: border-box;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  }

  .nav-left {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .hanko-circle {
    width: 14px;
    height: 14px;
    background-color: var(--hanko-red);
    border-radius: 50%;
    display: inline-block;
  }

  .nav-title {
    font-size: 0.95rem;
    font-weight: 700;
    letter-spacing: 0.12em;
    color: var(--ink-black);
  }

  .nav-level-tag {
    font-size: 0.72rem;
    font-family: monospace;
    background: var(--border-line);
    padding: 2px 7px;
    border-radius: 2px;
    color: var(--ink-secondary);
    font-weight: 600;
  }

  .nav-back-link {
    background: transparent;
    border: none;
    font-size: 0.9rem;
    color: var(--ink-secondary);
    cursor: pointer;
    padding: 4px 8px;
    font-weight: 500;
  }

  .nav-back-link:hover {
    color: var(--hanko-red);
  }

  .streak-badge {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 0.88rem;
    background: var(--paper-card);
    border: 1px solid var(--border-line-dark);
    padding: 5px 12px;
    border-radius: 3px;
    font-family: -apple-system, sans-serif;
  }

  .streak-fire {
    font-size: 0.95rem;
  }

  .streak-num {
    font-weight: 700;
    color: var(--hanko-red);
  }

  .streak-label {
    color: var(--ink-secondary);
    font-size: 0.82rem;
  }

  /* ================== ОБЩИЙ МАКЕТ КОНТЕНТА ================== */
  .content-view {
    width: 100%;
    max-width: 860px;
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    align-items: stretch;
  }

  .section-badge {
    align-self: flex-start;
    font-size: 0.78rem;
    color: var(--ink-faint);
    font-family: monospace;
    margin-bottom: 12px;
    letter-spacing: 0.04em;
  }

  .date-meta {
    font-size: 0.82rem;
    font-family: monospace;
    color: var(--ink-faint);
    margin-bottom: 4px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .page-title {
    font-size: 2.1rem;
    font-weight: 600;
    margin: 0;
    color: var(--ink-black);
    font-family: "Hiragino Mincho ProN", serif;
  }

  /* ================== ВАРИАНТ А: КНИЖНЫЙ РАЗВОРОТ (BOOKSPREAD) ================== */
  .dashboard-layout-spread {
    width: 100%;
    max-width: 860px;
  }

  .spread-header {
    margin-bottom: 24px;
  }

  .spread-subtitle {
    color: var(--ink-secondary);
    font-size: 0.95rem;
    margin: 6px 0 0 0;
    font-family: -apple-system, sans-serif;
  }

  .bookspread-board {
    background: var(--paper-card);
    border: 1px solid var(--border-line-dark);
    box-shadow: 0 4px 16px rgba(26, 24, 22, 0.04);
    display: flex;
    border-radius: 4px;
    margin-bottom: 24px;
    min-height: 290px;
  }

  /* Левая колонка со словами */
  .spread-words-col {
    flex: 1.6;
    padding: 28px 32px;
    border-right: 1px solid var(--border-line);
    display: flex;
    flex-direction: column;
  }

  .col-meta-header {
    display: flex;
    justify-content: space-between;
    font-size: 0.75rem;
    font-family: monospace;
    color: var(--ink-faint);
    margin-bottom: 18px;
    letter-spacing: 0.06em;
  }

  .kanji-matrix-grid {
    display: grid;
    grid-template-columns: repeat(5, 1fr);
    gap: 10px 8px;
    flex: 1;
    align-content: center;
  }

  .matrix-cell {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 10px 6px;
    background: var(--paper-bg);
    border: 1px solid var(--border-line);
    border-radius: 3px;
    box-sizing: border-box;
    transition: all 0.15s ease;
    overflow: hidden;
  }

  .matrix-cell:hover {
    border-color: var(--hanko-red);
    background: var(--paper-card);
    transform: translateY(-1px);
  }

  .m-kana {
    font-size: 0.65rem;
    color: var(--ink-faint);
    letter-spacing: 0.04em;
    white-space: nowrap;
  }

  .m-kanji {
    font-size: 1.35rem;
    font-weight: 500;
    color: var(--ink-black);
    font-family: "Hiragino Mincho ProN", serif;
    line-height: 1.25;
    margin: 3px 0;
    white-space: nowrap;
    text-overflow: ellipsis;
    letter-spacing: 0.02em;
  }

  .m-ru {
    font-size: 0.72rem;
    color: var(--ink-secondary);
    font-family: -apple-system, sans-serif;
    white-space: nowrap;
  }

  /* Правая колонка со статусом и кнопкой */
  .spread-action-col {
    flex: 1.05;
    padding: 28px 24px;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    background: rgba(247, 245, 240, 0.4);
  }

  .session-brief {
    display: flex;
    flex-direction: column;
    gap: 16px;
    font-family: -apple-system, sans-serif;
  }

  .brief-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-bottom: 8px;
    border-bottom: 1px dashed var(--border-line);
    font-size: 0.88rem;
  }

  .brief-label {
    color: var(--ink-secondary);
    font-size: 0.85rem;
  }

  .brief-val {
    font-family: monospace;
    font-size: 1.05rem;
    color: var(--ink-black);
  }

  /* Блок освоения колоды без светофора (чистый монохром) */
  .brief-item-mastery {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding-top: 4px;
  }

  .mastery-label-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .mini-progress-track {
    width: 100%;
    height: 4px;
    background: var(--border-line);
    border-radius: 2px;
    overflow: hidden;
  }

  .mini-progress-fill {
    height: 100%;
    background: var(--ink-black);
    border-radius: 2px;
  }

  .mastery-subtext {
    font-size: 0.75rem;
    color: var(--ink-faint);
    font-family: monospace;
  }

  .action-dock-box {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .spread-start-btn {
    width: 100%;
    padding: 14px 0;
    text-align: center;
    font-size: 0.95rem;
  }

  .hotkey-hint {
    font-size: 0.75rem;
    color: var(--ink-faint);
    text-align: center;
    font-family: monospace;
  }

  /* Нижняя строгая полоска ритма */
  .rhythm-strip {
    background: var(--paper-card);
    border: 1px solid var(--border-line-dark);
    padding: 14px 24px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    font-family: -apple-system, sans-serif;
  }

  .rhythm-title {
    font-size: 0.72rem;
    font-family: monospace;
    letter-spacing: 0.1em;
    color: var(--ink-faint);
    white-space: nowrap;
  }

  .rhythm-divider-vertical {
    width: 1px;
    height: 16px;
    background: var(--border-line);
    margin: 0 20px;
  }

  .rhythm-dots {
    display: flex;
    gap: 22px;
    font-size: 0.85rem;
  }

  .r-day {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--ink-secondary);
  }

  .r-day.done .r-mark {
    color: var(--hanko-red);
  }

  .r-day.wait .r-mark {
    color: var(--border-line-dark);
  }

  .r-day.today {
    font-weight: 700;
    color: var(--ink-black);
  }

  .r-day.today .r-mark {
    color: var(--hanko-red);
  }

  .action-btn-primary {
    background: var(--ink-black);
    color: #FFFFFF;
    border: 1px solid var(--ink-black);
    padding: 12px 28px;
    font-size: 0.95rem;
    font-weight: 600;
    cursor: pointer;
    border-radius: 3px;
    font-family: -apple-system, sans-serif;
    transition: all 0.15s ease;
  }

  .action-btn-primary:hover {
    background: var(--hanko-red);
    border-color: var(--hanko-red);
  }

  /* ================== СКРЫТЫЙ САЙДБАР (КНИЖНЫЙ РЕЕСТР) ================== */
  .sidebar-hover-zone {
    position: fixed;
    top: 0;
    left: 0;
    width: 24px;
    height: 100vh;
    z-index: 1000;
  }

  .sidebar-panel {
    position: fixed;
    top: 0;
    left: -280px;
    width: 270px;
    height: 100vh;
    background: var(--paper-card);
    border-right: 1px solid var(--border-line-dark);
    box-shadow: 12px 0 35px rgba(26, 24, 22, 0.06);
    display: flex;
    flex-direction: column;
    padding: 32px 20px;
    box-sizing: border-box;
    transition: transform 0.3s cubic-bezier(0.16, 1, 0.3, 1);
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  }

  .sidebar-hover-zone:hover .sidebar-panel,
  .sidebar-panel:hover {
    transform: translateX(280px);
  }

  .sidebar-brand {
    display: flex;
    align-items: center;
    gap: 12px;
    padding-bottom: 24px;
    border-bottom: 1px solid var(--border-line);
    margin-bottom: 24px;
  }

  .brand-hanko {
    color: var(--hanko-red);
    font-size: 1.6rem;
    line-height: 1;
  }

  .brand-text {
    display: flex;
    flex-direction: column;
  }

  .brand-jp {
    font-weight: 700;
    font-size: 1.1rem;
    color: var(--ink-black);
    font-family: "Hiragino Mincho ProN", serif;
  }

  .brand-sub {
    font-size: 0.75rem;
    color: var(--ink-faint);
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .sidebar-nav {
    display: flex;
    flex-direction: column;
    gap: 4px;
    flex: 1;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 10px 12px;
    border: none;
    background: transparent;
    color: var(--ink-secondary);
    font-size: 0.88rem;
    cursor: pointer;
    text-align: left;
    border-radius: 4px;
    transition: all 0.15s ease;
  }

  .nav-item:hover {
    background: var(--paper-bg);
    color: var(--ink-black);
  }

  .nav-item.active {
    background: var(--paper-bg);
    color: var(--hanko-red);
    border-left: 3px solid var(--hanko-red);
    font-weight: 600;
  }

  .nav-idx {
    font-family: monospace;
    font-size: 0.8rem;
    color: var(--ink-faint);
  }

  /* ================== РЕЖИМ УРОКА (ДВЕ РАЗДЕЛЬНЫЕ КАРТЫ) ================== */
  .lesson-meta-bar {
    width: 100%;
    margin-bottom: 28px;
  }

  .session-progress-bar {
    width: 100%;
    height: 3px;
    background: var(--border-line);
    margin-bottom: 8px;
    border-radius: 2px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: var(--hanko-red);
    transition: width 0.3s ease;
  }

  .progress-label-row {
    display: flex;
    justify-content: space-between;
    font-size: 0.8rem;
    color: var(--ink-faint);
    font-family: monospace;
  }

  .esc-hint {
    color: var(--ink-faint);
  }

  .kbd-pill {
    background: var(--paper-bg);
    border: 1px solid var(--border-line-dark);
    padding: 1px 6px;
    border-radius: 3px;
    font-family: monospace;
    font-size: 0.72rem;
    color: var(--ink-secondary);
  }

  .example-label {
    font-size: 0.72rem;
    color: var(--ink-faint);
    font-family: monospace;
    margin-bottom: 4px;
    text-transform: uppercase;
  }

  /* Финальная карточка завершения урока */
  .paper-completion-card {
    background: var(--paper-card);
    border: 1px solid var(--border-line-dark);
    box-shadow: 0 4px 20px rgba(26, 24, 22, 0.04);
    padding: 48px 40px;
    text-align: center;
    border-radius: 4px;
    max-width: 500px;
    margin: 40px auto 0 auto;
    font-family: -apple-system, sans-serif;
  }

  .completion-title {
    font-size: 1.6rem;
    color: var(--ink-black);
    margin: 0 0 10px 0;
    font-family: "Hiragino Mincho ProN", serif;
  }

  .completion-desc {
    color: var(--ink-secondary);
    font-size: 0.92rem;
    line-height: 1.5;
    margin: 0 0 28px 0;
  }

  .completion-stats {
    display: flex;
    justify-content: center;
    gap: 40px;
    padding: 16px 0;
    border-top: 1px dashed var(--border-line);
    border-bottom: 1px dashed var(--border-line);
    margin-bottom: 28px;
  }

  .c-stat {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .c-val {
    font-size: 1.8rem;
    font-weight: 700;
    font-family: monospace;
    color: var(--ink-black);
  }

  .c-label {
    font-size: 0.78rem;
    color: var(--ink-faint);
  }

  .completion-btn {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 12px 28px;
  }

  .btn-sub-kbd {
    font-family: monospace;
    font-size: 0.75rem;
    opacity: 0.7;
  }

  .cards-stage {
    display: flex;
    align-items: stretch;
    justify-content: center;
    width: 100%;
    gap: 0px;
    transition: gap 0.4s cubic-bezier(0.16, 1, 0.3, 1);
  }

  .cards-stage.revealed {
    gap: 24px;
  }

  /* Строгая бумажная карточка (чистая бумага, чернила, тонкая окантовка) */
  .paper-card {
    background: var(--paper-card);
    border: 1px solid var(--border-line-dark);
    box-shadow: 0 4px 16px rgba(26, 24, 22, 0.04);
    box-sizing: border-box;
    padding: 36px 32px;
    display: flex;
    flex-direction: column;
    justify-content: center;
    position: relative;
    border-radius: 4px;
  }

  .card-tag {
    position: absolute;
    top: 14px;
    left: 18px;
    font-size: 0.72rem;
    color: var(--ink-faint);
    font-family: -apple-system, sans-serif;
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }

  .card-top-row {
    position: absolute;
    top: 14px;
    left: 18px;
    right: 18px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    z-index: 2;
  }

  .card-top-row .card-tag {
    position: static;
  }

  .card-audio-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    background: transparent;
    border: 1px solid var(--border-line-dark);
    padding: 3px 8px;
    border-radius: 3px;
    color: var(--ink-secondary);
    font-size: 0.72rem;
    font-family: monospace;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .card-audio-btn:hover {
    color: var(--hanko-red);
    border-color: var(--hanko-red);
    background: rgba(200, 50, 38, 0.04);
  }

  .audio-btn-kbd {
    font-size: 0.7rem;
    opacity: 0.75;
  }

  .dict-mini-audio-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: 1px solid var(--border-line);
    color: var(--ink-faint);
    width: 20px;
    height: 20px;
    border-radius: 3px;
    cursor: pointer;
    padding: 0;
    margin-left: 6px;
    vertical-align: middle;
    transition: all 0.15s ease;
  }

  .dict-mini-audio-btn:hover {
    color: var(--hanko-red);
    border-color: var(--hanko-red);
    background: rgba(200, 50, 38, 0.06);
  }

  /* Левая карточка (Японский) */
  .card-japanese {
    width: 380px;
    min-height: 320px;
    align-items: center;
    text-align: center;
    cursor: pointer;
    transition: width 0.4s ease;
  }

  .cards-stage.revealed .card-japanese {
    width: 360px;
    cursor: default;
  }

  .furigana-top {
    font-size: 1.25rem;
    color: var(--ink-secondary);
    margin-bottom: 8px;
    letter-spacing: 0.08em;
  }

  .kanji-display {
    font-size: 4.2rem;
    margin: 0;
    font-weight: 500;
    color: var(--ink-black);
    line-height: 1.1;
    font-family: "Hiragino Mincho ProN", serif;
  }

  .reveal-hint {
    margin-top: 24px;
    font-size: 0.8rem;
    color: var(--hanko-red);
    font-family: -apple-system, sans-serif;
  }

  /* Правая карточка (Значение и контекст) */
  .card-translation {
    width: 440px;
    min-height: 320px;
    text-align: left;
    animation: slidePaperRight 0.35s cubic-bezier(0.16, 1, 0.3, 1) forwards;
  }

  @keyframes slidePaperRight {
    from { opacity: 0; transform: translateX(20px); }
    to { opacity: 1; transform: translateX(0); }
  }

  .translation-title {
    font-size: 2rem;
    margin: 0 0 20px 0;
    font-weight: 600;
    color: var(--ink-black);
    font-family: -apple-system, sans-serif;
  }

  .example-box {
    border-top: 1px solid var(--border-line);
    padding-top: 18px;
  }

  .example-jp {
    font-size: 1.15rem;
    color: var(--ink-black);
    margin-bottom: 6px;
    line-height: 1.4;
  }

  .example-target {
    font-size: 0.95rem;
    color: var(--ink-secondary);
    font-family: -apple-system, sans-serif;
  }

  /* Панель кнопок (строгие прямоугольные клавиши) */
  .review-dock {
    display: flex;
    gap: 12px;
    margin-top: 32px;
    opacity: 0;
    pointer-events: none;
    transition: opacity 0.3s ease;
    font-family: -apple-system, sans-serif;
  }

  .review-dock.visible {
    opacity: 1;
    pointer-events: auto;
  }

  .review-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 20px;
    font-size: 0.88rem;
    border: 1px solid var(--border-line-dark);
    background: var(--paper-card);
    color: var(--ink-black);
    cursor: pointer;
    border-radius: 3px;
    transition: all 0.15s ease;
  }

  .btn-kbd {
    border: 1px solid var(--border-line-dark);
    padding: 1px 6px;
    font-size: 0.72rem;
    font-family: monospace;
    color: var(--ink-faint);
    border-radius: 2px;
  }

  .btn-again:hover {
    border-color: var(--hanko-red);
    color: var(--hanko-red);
  }

  .btn-hard:hover {
    border-color: #D97706;
    color: #D97706;
  }

  .btn-good:hover {
    border-color: #16A34A;
    color: #16A34A;
  }

  /* ================== СПРИНТ ================== */
  .paper-blitz-box {
    width: 100%;
    max-width: 520px;
    background: var(--paper-card);
    border: 1px solid var(--border-line-dark);
    padding: 40px;
    box-sizing: border-box;
    text-align: center;
    border-radius: 4px;
  }

  .blitz-meta-line {
    display: flex;
    justify-content: space-between;
    padding-bottom: 20px;
    border-bottom: 1px solid var(--border-line);
    margin-bottom: 28px;
    font-family: monospace;
    font-size: 0.95rem;
  }

  .blitz-kanji {
    font-size: 4rem;
    margin-bottom: 4px;
    font-family: "Hiragino Mincho ProN", serif;
  }

  .blitz-kana {
    color: var(--ink-secondary);
    font-size: 1.2rem;
    margin-bottom: 32px;
  }

  .blitz-choice-row {
    display: flex;
    gap: 12px;
    justify-content: center;
  }

  /* ================== ГРАММАТИКА ================== */
  .grammar-layout {
    width: 100%;
    max-width: 860px;
  }

  .grammar-header-row {
    width: 100%;
    margin-bottom: 24px;
  }

  .grammar-subtitle {
    color: var(--ink-secondary);
    font-size: 0.95rem;
    margin: 6px 0 0 0;
    font-family: -apple-system, sans-serif;
  }

  .grammar-toolbar {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 18px;
    margin-bottom: 24px;
    font-family: -apple-system, sans-serif;
  }

  /* Каталожные вкладки в стиле японского оглавления (目次) */
  .grammar-index-nav {
    display: flex;
    gap: 28px;
    border-bottom: 1px solid var(--border-line-dark);
    padding: 0;
  }

  .index-tab {
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    padding: 8px 4px 14px 4px;
    margin-bottom: -1px;
    color: var(--ink-secondary);
    cursor: pointer;
    display: inline-flex;
    align-items: baseline;
    gap: 8px;
    transition: all 0.15s ease;
    font-family: -apple-system, sans-serif;
  }

  .index-tab:hover {
    color: var(--ink-black);
  }

  .index-tab.active {
    color: var(--ink-black);
    border-bottom-color: var(--hanko-red);
  }

  .tab-jp {
    font-family: "Hiragino Mincho ProN", serif;
    font-size: 0.95rem;
    font-weight: 600;
    color: var(--ink-black);
  }

  .index-tab:not(.active) .tab-jp {
    color: var(--ink-secondary);
  }

  .tab-ru {
    font-size: 0.85rem;
  }

  .tab-count {
    font-family: monospace;
    font-size: 0.72rem;
    color: var(--ink-faint);
  }

  .index-tab.active .tab-count {
    color: var(--hanko-red);
    font-weight: 700;
  }

  /* Строгая плашка поиска */
  .grammar-search-wrap {
    display: flex;
    align-items: center;
    gap: 14px;
    background: var(--paper-card);
    border: 1px solid var(--border-line-dark);
    padding: 6px 14px;
    border-radius: 3px;
    box-shadow: 0 1px 4px rgba(26, 24, 22, 0.02);
  }

  .search-tag {
    font-family: monospace;
    font-size: 0.72rem;
    color: var(--ink-faint);
    white-space: nowrap;
    letter-spacing: 0.05em;
    padding-right: 12px;
    border-right: 1px solid var(--border-line);
  }

  .grammar-search-box {
    position: relative;
    flex: 1;
    display: flex;
    align-items: center;
  }

  .grammar-search-input {
    width: 100%;
    box-sizing: border-box;
    padding: 6px 28px 6px 4px;
    background: transparent;
    border: none;
    font-size: 0.9rem;
    color: var(--ink-black);
    font-family: -apple-system, sans-serif;
    outline: none;
  }

  .search-clear-btn {
    position: absolute;
    right: 4px;
    top: 50%;
    transform: translateY(-50%);
    background: transparent;
    border: none;
    color: var(--ink-faint);
    cursor: pointer;
    font-size: 0.85rem;
    padding: 4px;
  }

  .search-clear-btn:hover {
    color: var(--ink-black);
  }

  .grammar-table-card {
    width: 100%;
    background: var(--paper-card);
    border: 1px solid var(--border-line-dark);
    border-radius: 4px;
    box-shadow: 0 4px 16px rgba(26, 24, 22, 0.03);
    margin-bottom: 40px;
  }

  .grammar-not-found {
    padding: 56px 24px;
    text-align: center;
    color: var(--ink-secondary);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
  }

  .grammar-entry {
    display: flex;
    gap: 24px;
    padding: 28px 32px;
    border-bottom: 1px solid var(--border-line);
  }

  .grammar-entry:last-child {
    border-bottom: none;
  }

  .entry-index {
    font-family: monospace;
    font-size: 1.1rem;
    color: var(--hanko-red);
    font-weight: 700;
    min-width: 24px;
  }

  .entry-content {
    flex: 1;
  }

  .entry-top-meta {
    margin-bottom: 6px;
  }

  .entry-cat-badge {
    font-size: 0.72rem;
    font-family: monospace;
    background: var(--paper-bg);
    border: 1px solid var(--border-line);
    padding: 2px 7px;
    border-radius: 2px;
    color: var(--ink-faint);
    font-weight: 600;
    letter-spacing: 0.04em;
  }

  .entry-content h3 {
    margin: 4px 0 8px 0;
    font-size: 1.25rem;
    font-family: "Hiragino Mincho ProN", serif;
    color: var(--ink-black);
  }

  .formula-line {
    font-family: -apple-system, sans-serif;
    font-size: 0.88rem;
    color: var(--ink-secondary);
    margin: 4px 0 12px 0;
  }

  .formula-line code {
    background: var(--paper-bg);
    padding: 3px 8px;
    border: 1px solid var(--border-line-dark);
    border-radius: 2px;
    color: var(--ink-black);
    font-weight: 600;
  }

  .entry-desc {
    font-family: -apple-system, sans-serif;
    font-size: 0.92rem;
    color: var(--ink-secondary);
    line-height: 1.6;
    margin: 0 0 16px 0;
  }

  .examples-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
    margin-top: 12px;
  }

  .entry-example {
    font-size: 0.95rem;
    padding-left: 14px;
    border-left: 2px solid var(--border-line-dark);
    display: flex;
    flex-direction: column;
    gap: 3px;
    background: rgba(247, 245, 240, 0.4);
    padding: 8px 12px;
    border-radius: 0 3px 3px 0;
  }

  .example-jp-line {
    display: flex;
    gap: 10px;
    align-items: baseline;
  }

  .ex-jp {
    font-weight: 600;
    color: var(--ink-black);
    font-family: "Hiragino Mincho ProN", serif;
    font-size: 1.05rem;
  }

  .ex-kana {
    font-size: 0.78rem;
    color: var(--ink-faint);
    font-family: monospace;
  }

  .ex-ru {
    font-size: 0.88rem;
    color: var(--ink-secondary);
    font-family: -apple-system, sans-serif;
  }

  /* ================== СЛОВАРЬ (5.0 単語帳) ================== */
  .dict-layout {
    width: 100%;
    max-width: 860px;
  }

  .dict-header-row {
    width: 100%;
    margin-bottom: 24px;
  }

  .dict-subtitle {
    color: var(--ink-secondary);
    font-size: 0.95rem;
    margin: 6px 0 0 0;
    font-family: -apple-system, sans-serif;
  }

  .dict-toolbar {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 18px;
    margin-bottom: 24px;
    font-family: -apple-system, sans-serif;
  }

  .dict-index-nav {
    display: flex;
    gap: 28px;
    border-bottom: 1px solid var(--border-line-dark);
    padding: 0;
  }

  .dict-controls-row {
    display: flex;
    gap: 16px;
    align-items: center;
    width: 100%;
  }

  .dict-search-wrap {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 14px;
    background: var(--paper-card);
    border: 1px solid var(--border-line-dark);
    padding: 6px 14px;
    border-radius: 3px;
    box-shadow: 0 1px 4px rgba(26, 24, 22, 0.02);
  }

  .dict-sort-wrap {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.85rem;
    color: var(--ink-secondary);
    white-space: nowrap;
  }

  .sort-label {
    font-family: monospace;
    font-size: 0.75rem;
    color: var(--ink-faint);
  }

  .dict-sort-select {
    background: var(--paper-card);
    border: 1px solid var(--border-line-dark);
    padding: 7px 12px;
    border-radius: 3px;
    font-size: 0.85rem;
    color: var(--ink-black);
    outline: none;
    font-family: -apple-system, sans-serif;
    cursor: pointer;
  }

  .dict-table-card {
    width: 100%;
    background: var(--paper-card);
    border: 1px solid var(--border-line-dark);
    border-radius: 4px;
    box-shadow: 0 4px 16px rgba(26, 24, 22, 0.03);
    margin-bottom: 40px;
  }

  .dict-entry {
    display: flex;
    align-items: center;
    gap: 20px;
    padding: 20px 24px;
    border-bottom: 1px solid var(--border-line);
    transition: background-color 0.15s ease;
  }

  .dict-entry:hover {
    background-color: rgba(247, 245, 240, 0.5);
  }

  .dict-entry:last-child {
    border-bottom: none;
  }

  .dict-index {
    font-family: monospace;
    font-size: 0.85rem;
    color: var(--ink-faint);
    min-width: 26px;
  }

  .dict-main-col {
    min-width: 130px;
    display: flex;
    flex-direction: column;
  }

  .dict-kana-line {
    font-size: 0.72rem;
    color: var(--ink-faint);
    letter-spacing: 0.04em;
  }

  .dict-kanji-line {
    font-size: 1.45rem;
    font-weight: 500;
    color: var(--ink-black);
    font-family: "Hiragino Mincho ProN", serif;
    line-height: 1.2;
  }

  .dict-trans-col {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .dict-translation {
    font-size: 1.05rem;
    font-weight: 600;
    color: var(--ink-black);
    font-family: -apple-system, sans-serif;
  }

  .dict-example {
    font-size: 0.85rem;
    color: var(--ink-secondary);
    display: flex;
    gap: 6px;
    align-items: baseline;
    flex-wrap: wrap;
  }

  .ex-label {
    font-family: monospace;
    font-size: 0.72rem;
    color: var(--hanko-red);
    font-weight: 700;
  }

  .ex-sep {
    color: var(--border-line-dark);
  }

  .dict-meta-col {
    display: flex;
    justify-content: flex-end;
    min-width: 120px;
  }

  .srs-badge {
    font-size: 0.75rem;
    font-family: monospace;
    padding: 3px 8px;
    border-radius: 2px;
    white-space: nowrap;
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }

  .badge-new {
    background: var(--paper-bg);
    border: 1px solid var(--border-line);
    color: var(--ink-faint);
  }

  .badge-repeat {
    background: rgba(200, 50, 38, 0.08);
    border: 1px solid var(--hanko-red);
    color: var(--hanko-red);
    font-weight: 600;
  }

  .badge-learned {
    background: rgba(22, 163, 74, 0.08);
    border: 1px solid #16A34A;
    color: #16A34A;
    font-weight: 600;
  }

  .badge-fire {
    font-size: 0.8rem;
  }

  /* ================== НАСТРОЙКИ ================== */
  .settings-table {
    width: 100%;
    background: var(--paper-card);
    border: 1px solid var(--border-line-dark);
    border-radius: 4px;
    font-family: -apple-system, sans-serif;
  }

  .settings-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 24px 28px;
    border-bottom: 1px solid var(--border-line);
  }

  .settings-item:last-child {
    border-bottom: none;
  }

  .settings-item p {
    color: var(--ink-secondary);
    font-size: 0.85rem;
    margin: 4px 0 0 0;
  }

  .paper-select, .paper-input {
    padding: 8px 14px;
    border: 1px solid var(--border-line-dark);
    background: var(--paper-bg);
    color: var(--ink-black);
    font-size: 0.9rem;
    border-radius: 2px;
  }

  .seal-mark {
    width: 64px;
    height: 64px;
    border: 2px solid var(--hanko-red);
    color: var(--hanko-red);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 2.2rem;
    margin: 0 auto 20px auto;
    font-family: "Hiragino Mincho ProN", serif;
  }

  /* Кнопка настроек в правом нижнем углу */
  .bottom-settings-btn {
    position: fixed;
    bottom: 24px;
    right: 28px;
    width: 40px;
    height: 40px;
    border-radius: 50%;
    background: var(--paper-card);
    border: 1px solid var(--border-line-dark);
    color: var(--ink-faint);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    box-shadow: 0 2px 8px rgba(26, 24, 22, 0.06);
    transition: all 0.2s ease;
    z-index: 100;
  }

  .bottom-settings-btn:hover {
    color: var(--ink-black);
    border-color: var(--ink-black);
    transform: rotate(30deg);
  }

  .bottom-settings-btn.active {
    background: var(--ink-black);
    color: #FFFFFF;
    border-color: var(--ink-black);
  }
</style>
