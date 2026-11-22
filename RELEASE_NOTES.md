## NyashkAir 1.0.13

- Спектри входу й виходу тепер плавні: крива між бінами FFT замість прямих відрізків і кутів.

## NyashkAir 1.0.12

- Більше жодних випадкових тріщань через інші копії плагіна: в аудіопотоці бралося глобальне блокування (лишилося від демо-режиму для скріншотів), спільне для всіх копій. Коли Bitwig обробляв кілька доріжок паралельно, один аудіопотік міг чекати на інший і не встигати. Тепер перевірка без блокувань.

## NyashkAir 1.0.11

- Кілька копій плагіна в одному проєкті тепер відкриваються миттєво: раніше друге вікно відкривалося до 1,5 с, а закриття будь-якого вікна блокувало DAW до 1 с (вікно чекало, поки закриються всі вікна наших плагінів у процесі, а не лише своє). Через це в Bitwig перше вікно могло на секунду зникати.

## NyashkAir 1.0.10

- Нові кнопки: клавіша сидить у заглибленні панелі, опукла матова, при ввімкненні світиться зсередини, а світло м'яко лягає на панель навколо (раніше світіння обрізалось прямокутною рамкою).

## NyashkAir 1.0.9

- Шкали, риски, LED і підписи тепер згладжені (малюються в 4 рази більшими й зменшуються), без кривих країв.
- Світіння адитивне, як у справжніх LED: м'який ореол і яскраве ядро.
- Плавні анімації: кнопки загоряються й гаснуть, LED-кільця ковзають за ручкою, обідок навколо ручки плавно проявляється.
- Оптимізація: оновлення екрана 30 разів на секунду (Opto було 60), анімації не додають зайвих перемальовувань.

## NyashkAir 1.0.8

- Деталі дизайну: стрілка ручки починається від краю ковпачка (як справжня гравіровка), бокові стінки металевих ручок ловлять світло, навколо ручки, яку крутиш або на яку наводиш, світиться тонкий обідок, LED-кільця без зайвих рисок на маленьких ручках, мітки пікового рівня на LED-шкалах, більші підписи.

## NyashkAir 1.0.7

- Новий дизайн: сучасне аналогове залізо. Корпус, ручки, кнопки й екрани відрендерені як справжні фізичні об'єкти: точені алюмінієві ручки з рифленою стінкою, тіні в три шари, анодований метал з подряпинами, пилом і потертими краями, кнопки з підсвіткою, LED-дуги навколо ручок, скло на екранах.
- Ручки: тягни вгору / вниз (Shift — точно), колесо миші, подвійний клік — значення за замовчуванням, правий клік — ввести число. Значення показується при наведенні.

## NyashkAir 1.0.6

- Оверсемплінг гармонік без втрат на верху і з меншим аліасингом (новий поліфазний half-band фільтр).
- Захист від NaN / inf від хоста: один «битий» семпл більше не ламає плагін назавжди (раніше фільтри залишались у NaN і звук пропадав до перезапуску).

## NyashkAir 1.0.5

- **The sound now matches the curve:** the harmonic paths also passed a phase-shifted copy of the band itself, which bent the real response away from the drawn curve (Mid Air at 50 % even dipped slightly at 8 kHz while the graph showed +4 dB). They now add only the new harmonics; the curve and the sound agree within 1.5 dB at 44.1 / 48 / 96 kHz. Full Mid Air is now +6 dB at 5 kHz, full High Air +10 dB at 14 kHz.
- The ADDED readout averages over ~300 ms instead of a few samples (steadier).
- New behaviour tests: each knob lifts its own region steadily, lows stay within 0.5 dB, loud highs get less boost, no audible aliasing from the exciter (worst fold-back -75 dB), trim exact.

## NyashkAir 1.0.4

- Level scale shows a plain 0 instead of +0.

## NyashkAir 1.0.3

- **Alt-tab fix (Linux):** after alt-tab, minimising or covering the window, parts of the picture (backgrounds, knobs, images) could stay blank until something on them moved. The window is now drawn again in full whenever the system throws its contents away, and pictures are no longer lost while the window is hidden or a frame is skipped.
- Smoother images in the CPU renderer (used when Vulkan is not available): pictures drawn smaller than their real size, such as knobs and photos, are now scaled with mipmaps instead of picking single pixels, so they no longer look grainy or jagged.

## NyashkAir 1.0.2

- **Display fix:** the built-in CPU renderer (used when Vulkan is not available) drew pixels on the shared edges of triangles twice, which showed as brighter seams and specks in see-through fills (spectra, curves, bars). It now uses an exact fill rule like a GPU: every pixel is drawn once, without holes.
- The frequency grid stays visible over the live spectrum.
- Value boxes have a small gap between them, and a value that would not fit its box is drawn smaller instead of running over the frame.
- Clicking a value box and pressing Enter no longer changes a value whose unit matters (for example "1.03 s" was read back as 1.03 ms).
- New automatic test: every parameter's text is typed back over its whole range and must give the same value.

## NyashkAir 1.0.1

- **Live spectrum:** the input (thin line) and output (filled) spectrum now run behind the air curve, so you see exactly what the plugin adds to the top end.

## NyashkAir 1.0.0

First release. VST3 for Linux, Windows and macOS.
