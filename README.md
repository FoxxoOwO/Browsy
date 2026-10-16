# Browsy 🌐

Moderní, rychlý a bezpečný webový prohlížeč napsaný v **Rustu** postavený na jádře **Servo**.

---

## 🚀 Hlavní Vlastnosti & Implementované Požadavky

### 1. Webové jádro Servo
- Využívá nejnovější **Servo Web Engine** (`servo 0.7.0`, `servo-embedder-traits 0.7.0`).
- Paralelní vykreslovací architektura v Rustu s plnou podporou Web Standards.
- Integrované grafické rozhraní postavené na **egui / eframe (0.37)** s hardwarovou akcelerací.

### 2. Integrovaný uBlock Origin (Ad-blocker)
- Postaveno na syntaktickém enginu Brave (`adblock 0.13.3`) s podporou EasyListu a uBlock Origin pravidel.
- **Síťové filtrování:** Automatické blokování sledovacích a reklamních požadavků (skripty, obrázky, iframy, analytika) v reálném čase.
- **Kosmetické filtrování (Cosmetic Filtering):** Dynamická injekce CSS selektorů (`display: none !important;`) pro skrytí reklamních ploch v DOMu stránky.
- Možnost vypnutí blokování pro konkrétní weby (Allowlist) a počítadlo zablokovaných prvků na štítu prohlížeče.

### 3. Integrovaný Pop-up Blocker
- Zabraňuje otevírání nevyžádaných oken a přesměrování ze skriptů třetích stran.
- **Heuristika uživatelské aktivace:** Povoluje otevření nového okna pouze v případě, že bylo vyvoláno prokazatelným gestem uživatele (kliknutí myší, stisk klávesy v časovém okně 1500 ms).
- Neintruzivní notifikační lišta při zablokování s možností jednorázového otevření nebo trvalého povolení domény (pro OAuth / přihlašovací dialogy).

### 4. Stahování videí z webu (Video Downloader)
- **Automatický detektor médií:** Síťový a DOM sniffer zachytává streamy a soubory formátů **MP4**, **WebM**, **HLS playlisty (`.m3u8`)**, **DASH manifesty (`.mpd`)** a zvukové stopy.
- **Integrovaný downloader:** Podpora přímého vícevláknového stahování HTTP s měřením rychlosti, průběhu a velikosti souboru.
- **Hook pro yt-dlp:** Možnost stažení a konverze z pokročilých video platforem jedním kliknutím.

### 5. Text Expander
- Automatické rozbalování textových zkratek přímo při psaní:
  - Vestavěné zkratky: `:email`, `:date`, `:time`, `:shrug` (¯\\_(ツ)_/¯), `:br`, `:lipsum` a dynamické šablony (`{date}`, `{time}`, `{datetime}`).
  - Možnost definice vlastních libovolných zkratek a expanzí v nastavení.
- **Dvojí integrace:**
  1. V adresním řádku prohlížeče (Omnibox).
  2. Přímo ve webových formulářích, textových polích (`<input>`, `<textarea>`) a `contenteditable` editorech prostřednictvím automaticky injektovaného content scriptu (`browsy_text_expander.js`).

### 6. Podpora doplňků z Firefoxu a Chrome (WebExtensions)
- Podpora pro rozšíření podle specifikací **Manifest V2** i **Manifest V3** (`manifest.json`).
- Plnohodnotný match pattern parser pro URL pravidla (`<all_urls>`, `*://*.example.com/*`).
- JavaScript polyfill vrstva zajišťující kompatibilitu objektů `window.chrome` i `window.browser` (`runtime`, `storage.local`, `tabs`, `action`).
- Správce doplňků pro načítání rozbalených doplňků ze složky (`browsy://extensions`).

### 7. Vertikální i Horizontální Taby
- **Okamžité přepínání rozvržení:** Zkratkou `Ctrl+Shift+V` nebo tlačítkem v liště.
- **Horizontální režim:** Klasické karty v horní liště.
- **Vertikální režim:** Moderní postranní panel ve stylu Arc Browseru nebo MS Edge s možností sbalení.
- Podpora připínání tabů (Pinned tabs), indikace načítání, odznáčky počtu zablokovaných reklam a detekovaných videí.

---

## ⌨️ Klávesové Zkratky

| Zkratka | Akce |
|---------|------|
| `Ctrl + T` | Otevřít nový tab |
| `Ctrl + W` | Zavřít aktivní tab |
| `Ctrl + R` / `F5` | Obnovit stránku |
| `Ctrl + L` | Aktivovat adresní řádek |
| `Ctrl + Shift + V` | Přepnout mezi vertikálními a horizontálními taby |
| `Ctrl + Shift + E` | Otevřít správce Text Expanderu |
| `Alt + Left` | Zpět v historii |
| `Alt + Right` | Vpřed v historii |

---

## 🛠️ Struktura Projektu

```
browsy/
├── Cargo.toml
├── build.rs               # Build skript + MSVC STL kompatibilní shim
├── src/
│   ├── main.rs            # Vstupní bod aplikace
│   ├── lib.rs             # Export modulů
│   ├── app.rs             # eframe::App stav a hlavní UI smyčka
│   ├── compat_stl.c       # Kompatibilní MSVC STL fallback pro mozjs_sys
│   ├── adblock/           # uBlock Origin / Brave adblock engine
│   ├── popup/             # Detekce a blokování vyskakovacích oken
│   ├── video_downloader/  # Síťový sniffer a stahovač médií
│   ├── text_expander/     # Text expander a content script
│   ├── extensions/        # WebExtensions runtime, manifest parser a polyfill
│   ├── tabs/              # Správa vertikálních a horizontálních tabů
│   ├── engine/            # Integrace jádra Servo a WebViewDelegate
│   └── ui/                # eframe/egui komponenty a dialogy
└── tests/                 # Automatizované unit a integrační testy
```

---

## 🚀 Sestavení a Spuštění

### Požadavky
- Rust toolchain (2024 edition)
- Windows s MSVC Build Tools / Linux / macOS

### Spuštění testů:
```bash
cargo test
```

### Sestavení a spuštění prohlížeče:
```bash
cargo run
```
