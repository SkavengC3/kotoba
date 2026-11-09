<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  interface Japanese {
    origin: string;
    kana: string;
  }

  interface Translation {
    origin: string;
    transcription: string;
  }

  interface Example {
    japanese: string;
    other: Record<string, string>;
  }

  interface Card {
    id: number;
    japanese: Japanese;
    translation: Record<string, Translation>;
    examples: Example[];
  }

  type Screen = "dashboard" | "lesson" | "blitz" | "grammar" | "dictionary" | "settings";

  let currentScreen = $state<Screen>("dashboard");
  let currentLang = $state("Ru");

  let card = $state<Card | null>(null);
  let isRevealed = $state(false);
  let totalSessionCount = $state(10);
  let completedCount = $state(0);

  // Спринт (60 сек)
  let blitzTimer = $state(60);
  let blitzScore = $state(0);
  let blitzActive = $state(false);
  let blitzInterval: any = null;

  async function loadCard() {
    card = await invoke<Card | null>("get_current_card");
    isRevealed = false;
  }

  async function handleReview(button: "Instantly" | "Difficult" | "Repeat") {
    if (button !== "Repeat") {
      completedCount += 1;
    }
    card = await invoke<Card | null>("review", { button });
    isRevealed = false;
  }

  function flipCard() {
    if (!isRevealed) {
      isRevealed = true;
    }
  }

  function navigateTo(screen: Screen) {
    currentScreen = screen;
    if (screen === "lesson") {
      loadCard();
    }
    if (screen !== "blitz" && blitzActive) {
      stopBlitz();
    }
  }

  function startBlitz() {
    blitzActive = true;
    blitzTimer = 60;
    blitzScore = 0;
    loadCard();
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
    }
  }

  onMount(() => {
    loadCard();
    window.addEventListener("keydown", handleKeydown);
    return () => {
      window.removeEventListener("keydown", handleKeydown);
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
        <span class="streak-num">4</span>
        <span class="streak-label">дня подряд</span>
      </div>
    {/if}
  </header>

  <!-- ================== ЭКРАН 1: ГЛАВНЫЙ (ВАРИАНТ А: EDITORIAL BOOKSPREAD) ================== -->
  {#if currentScreen === "dashboard"}
    <div class="content-view dashboard-layout-spread">
      <!-- Заголовок дня -->
      <div class="spread-header">
        <div class="date-meta">9 ноября • 4 日目</div>
        <h1 class="page-title">Сегодняшний разворот</h1>
        <p class="spread-subtitle">10 выражений к освоению. Оценочное время: ~5 минут.</p>
      </div>

      <!-- Главный книжный разворот: Сетка слов слева + Панель действия справа -->
      <div class="bookspread-board">
        <!-- Левая часть: Реальная сетка иероглифов сегодняшней сессии -->
        <div class="spread-words-col">
          <div class="col-meta-header">
            <span>ПРЕВЬЮ СЛОВАРЯ (10)</span>
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
              <span class="brief-label">Новые слова</span>
              <strong class="brief-val">10</strong>
            </div>
            <div class="brief-item">
              <span class="brief-label">На повторении</span>
              <strong class="brief-val">0</strong>
            </div>
            <div class="brief-item-mastery">
              <div class="mastery-label-row">
                <span class="brief-label">Освоение колоды N5</span>
                <strong class="brief-val">10%</strong>
              </div>
              <div class="mini-progress-track">
                <div class="mini-progress-fill" style="width: 10%"></div>
              </div>
              <span class="mastery-subtext">10 из 100 базовых слов</span>
            </div>
          </div>

          <div class="action-dock-box">
            <button class="action-btn-primary spread-start-btn" onclick={() => navigateTo("lesson")}>
              Открыть урок →
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
          <span class="r-day done"><span class="r-mark">●</span> Пн</span>
          <span class="r-day done"><span class="r-mark">●</span> Вт</span>
          <span class="r-day done"><span class="r-mark">●</span> Ср</span>
          <span class="r-day done today"><span class="r-mark">●</span> Чт</span>
          <span class="r-day wait"><span class="r-mark">○</span> Пт</span>
          <span class="r-day wait"><span class="r-mark">○</span> Сб</span>
          <span class="r-day wait"><span class="r-mark">○</span> Вс</span>
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
            <div class="card-tag">単語 • Слово</div>
            <div class="furigana-top">{card.japanese.kana}</div>
            <h1 class="kanji-display">{card.japanese.origin}</h1>
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
                <div class="transcription-label">{card.translation[currentLang]?.transcription || ''}</div>
                <h2 class="translation-title">{card.translation[currentLang]?.origin || '—'}</h2>
              </div>

              {#if card.examples.length > 0}
                <div class="example-box">
                  <div class="example-label">例文 (Пример):</div>
                  <div class="example-jp">{card.examples[0].japanese}</div>
                  <div class="example-target">{card.examples[0].other[currentLang] || ''}</div>
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
              <span class="c-val">4</span>
              <span class="c-label">дня серия (стрик)</span>
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
            <button class="action-btn-primary" onclick={startBlitz}>Начать спринт</button>
          </div>
        {:else if !blitzActive && blitzTimer === 0}
          <div class="blitz-intro">
            <h2>Время истекло</h2>
            <p>Правильных ответов: <strong>{blitzScore}</strong></p>
            <button class="action-btn-primary" onclick={startBlitz}>Повторить</button>
          </div>
        {:else if card}
          <div class="blitz-word-stage">
            <div class="blitz-kanji">{card.japanese.origin}</div>
            <div class="blitz-kana">{card.japanese.kana}</div>

            <div class="blitz-choice-row">
              <button class="action-btn-primary" onclick={() => { blitzScore += 1; handleReview("Instantly"); }}>
                {card.translation[currentLang]?.origin || 'Знаю'}
              </button>
              <button class="action-btn-outline" onclick={() => handleReview("Repeat")}>
                Пропустить
              </button>
            </div>
          </div>
        {/if}
      </div>
    </div>

  <!-- ================== ЭКРАН 4: ГРАММАТИКА ================== -->
  {:else if currentScreen === "grammar"}
    <div class="content-view grammar-layout">
      <div class="section-badge">4.0 基本文法 — Базовые правила (N5)</div>
      <h1 class="page-title">Справочник грамматики</h1>

      <div class="grammar-table-card">
        <article class="grammar-entry">
          <div class="entry-index">01</div>
          <div class="entry-content">
            <h3>〜は (wa) — Тематическая частица</h3>
            <p class="formula-line"><code>Существительное + は</code></p>
            <p class="entry-desc">Выделяет тему высказывания. Часто переводится как «Что касается X, то...».</p>
            <div class="entry-example">
              <strong>私は学生です。</strong> (Watashi wa gakusei desu) — Я студент.
            </div>
          </div>
        </article>

        <article class="grammar-entry">
          <div class="entry-index">02</div>
          <div class="entry-content">
            <h3>〜てください (te kudasai) — Вежливая просьба</h3>
            <p class="formula-line"><code>Глагол в て-форме + ください</code></p>
            <p class="entry-desc">Стандартный способ вежливо попросить собеседника совершить действие.</p>
            <div class="entry-example">
              <strong>日本語で話してください。</strong> (Nihongo de hanashite kudasai) — Пожалуйста, говорите по-японски.
            </div>
          </div>
        </article>
      </div>
    </div>

  <!-- ================== ЭКРАН 5: МОЙ СЛОВАРЬ ================== -->
  {:else if currentScreen === "dictionary"}
    <div class="content-view dict-layout">
      <div class="section-badge">5.0 私の単語帳 — Личные записи</div>
      <h1 class="page-title">Мой словарь</h1>

      <div class="paper-empty-box">
        <p>Здесь будут собраны отмеченные вами слова и сложные иероглифы для прицельного повторения.</p>
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
          <select bind:value={currentLang} class="paper-select">
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
          <input type="number" min="5" max="50" value="10" class="paper-input" />
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
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 20px 0;
    border-bottom: 1px solid var(--border-line);
    margin-bottom: 36px;
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
    max-width: 820px;
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    align-items: stretch;
  }

  .dash-header {
    margin-bottom: 24px;
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

  /* ================== ОБЩИЙ МАКЕТ КОНТЕНТА ================== */
  .content-view {
    width: 100%;
    max-width: 860px;
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .section-badge {
    align-self: flex-start;
    font-size: 0.8rem;
    color: var(--ink-faint);
    font-family: monospace;
    margin-bottom: 12px;
  }

  .page-title {
    align-self: flex-start;
    font-size: 2.2rem;
    font-weight: 600;
    margin: 0 0 8px 0;
    color: var(--ink-black);
    font-family: "Hiragino Mincho ProN", serif;
  }

  .page-desc {
    align-self: flex-start;
    color: var(--ink-secondary);
    margin: 0 0 32px 0;
    font-size: 1rem;
    font-family: -apple-system, sans-serif;
  }

  /* ================== ГЛАВНЫЙ ЭКРАН ================== */
  .paper-dashboard-card {
    width: 100%;
    background: var(--paper-card);
    border: 1px solid var(--border-line-dark);
    box-shadow: 0 4px 16px rgba(26, 24, 22, 0.04);
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 44px;
    box-sizing: border-box;
    border-radius: 4px;
  }

  .status-indicator {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 1.15rem;
    font-family: -apple-system, sans-serif;
    color: var(--ink-black);
  }

  .accent-bar {
    width: 4px;
    height: 20px;
    background-color: var(--hanko-red);
    display: inline-block;
  }

  .card-subnote {
    color: var(--ink-secondary);
    font-size: 0.92rem;
    margin: 12px 0 28px 0;
    max-width: 480px;
    line-height: 1.5;
    font-family: -apple-system, sans-serif;
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

  .action-btn-outline {
    background: transparent;
    color: var(--ink-black);
    border: 1px solid var(--border-line-dark);
    padding: 10px 20px;
    font-size: 0.9rem;
    cursor: pointer;
    border-radius: 3px;
    font-family: -apple-system, sans-serif;
  }

  .action-btn-outline:hover {
    border-color: var(--ink-black);
  }

  .kanji-seal {
    width: 96px;
    height: 96px;
    border: 2px solid var(--border-line-dark);
    color: var(--ink-faint);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 3.2rem;
    font-family: "Hiragino Mincho ProN", serif;
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

  .transcription-label {
    font-size: 0.82rem;
    color: var(--ink-faint);
    text-transform: uppercase;
    font-family: -apple-system, sans-serif;
    margin-bottom: 2px;
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
  .grammar-table-card {
    width: 100%;
    background: var(--paper-card);
    border: 1px solid var(--border-line-dark);
    border-radius: 4px;
  }

  .grammar-entry {
    display: flex;
    gap: 24px;
    padding: 28px;
    border-bottom: 1px solid var(--border-line);
  }

  .grammar-entry:last-child {
    border-bottom: none;
  }

  .entry-index {
    font-family: monospace;
    font-size: 1rem;
    color: var(--hanko-red);
    font-weight: 700;
  }

  .entry-content h3 {
    margin: 0 0 6px 0;
    font-size: 1.2rem;
    font-family: "Hiragino Mincho ProN", serif;
  }

  .formula-line {
    font-family: -apple-system, sans-serif;
    font-size: 0.88rem;
    color: var(--ink-secondary);
    margin: 4px 0 10px 0;
  }

  .formula-line code {
    background: var(--paper-bg);
    padding: 2px 6px;
    border: 1px solid var(--border-line);
    border-radius: 2px;
  }

  .entry-desc {
    font-family: -apple-system, sans-serif;
    font-size: 0.92rem;
    color: var(--ink-secondary);
    line-height: 1.5;
    margin: 0 0 12px 0;
  }

  .entry-example {
    font-size: 0.98rem;
    padding-left: 12px;
    border-left: 2px solid var(--border-line-dark);
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

  .paper-empty-box {
    background: var(--paper-card);
    border: 1px solid var(--border-line-dark);
    padding: 48px;
    text-align: center;
    border-radius: 4px;
    width: 100%;
    box-sizing: border-box;
    font-family: -apple-system, sans-serif;
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
