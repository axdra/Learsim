# Learsim Radio Panel — PCB

KiCad project (`learsim pcbs.kicad_*`) for the Learsim flight-sim **COM/NAV radio
panel**: an ESP32-C6 drives two MAX7219 LED drivers that multiplex two 6-digit
7-segment displays, plus two rotary encoders, six mode buttons and per-button
SK6812 RGB backlights.

This document describes the **schematic wiring and supporting components** that were
added to connect the previously-placed (but unwired) parts.

## Block diagram

```
                +5V ── J1 ─┬─ ESP32-C6 Vsys/Vbus
                           ├─ MAX7219 U2/U3  V+
                           └─ SK6812 D1..D6  VDD
 ESP32-C6 (U1)
   GPIO6  ── SPI DIN ─▶ U2.DIN ─▶ U2.DOUT ─▶ U3.DIN     (daisy-chain)
   GPIO7  ── SPI CLK ─▶ U2.CLK, U3.CLK
   GPIO4  ── SPI LOAD▶ U2.LOAD, U3.LOAD
   GPIO21 ── LED DATA▶ D1 ▶ D2 ▶ D3 ▶ D4 ▶ D5 ▶ D6      (WS2812-style chain)
   GPIO15/16/5  ── Encoder SW1 (MHz)  A / B / push
   GPIO18/19/20 ── Encoder SW2 (KHz)  A / B / push
   EXIO1..6 (TCA9554) ── COM1 COM2 NAV1 NAV2 ADF1 Switch1 buttons

   U2 ─▶ DS2 (6-digit display)      U3 ─▶ DS1 (6-digit display)
```

## ESP32-C6 GPIO map

| Function            | ESP32-C6 pin | Net       |
|---------------------|--------------|-----------|
| MAX7219 SPI DIN     | GPIO6        | SPI_DIN   |
| MAX7219 SPI CLK     | GPIO7        | SPI_CLK   |
| MAX7219 SPI LOAD/CS | GPIO4        | SPI_LOAD  |
| SK6812 data out     | GPIO21       | (existing)|
| Encoder MHz A / B   | GPIO15 / 16  | ENC1_A/B  |
| Encoder MHz push    | GPIO5        | ENC1_SW   |
| Encoder KHz A / B   | GPIO18 / 19  | ENC2_A/B  |
| Encoder KHz push    | GPIO20       | ENC2_SW   |
| Buttons ×6          | EXIO1..EXIO6 | BTN_*     |
| SW3 toggle throw 1  | GPIO8        | SW3_A     |
| SW3 toggle throw 2  | GPIO9        | SW3_B     |
| Power in            | Vsys (+5V)   | +5V       |
| Logic rail          | 3V3          | +3V3      |

Spare, broken-out pins for expansion: TX, RX, GPIO14, EXIO7.

**SW3 (SPDT toggle).** Common pole (pin 2) → GND; each throw read on its own GPIO with a
10 kΩ pull-up to +3V3 (pin 1 → GPIO8 via R15, pin 3 → GPIO9 via R16). A throw reads LOW when
selected and HIGH otherwise, so firmware can distinguish both positions (and a centre-OFF
state if the part has one).

## MAX7219 ↔ display wiring (common-cathode)

Each MAX7219 drives one display. `U2 → DS2`, `U3 → DS1`. Segment lines are the
display **anodes**, digit lines are the **cathodes**.

| MAX7219 | Display pin | | MAX7219 | Display pin |
|---------|-------------|-|---------|-------------|
| SEG_A (14) | A (11) | | DIG0 (2)  | DIG1 (14) |
| SEG_B (16) | B (7)  | | DIG1 (11) | DIG2 (13) |
| SEG_C (20) | C (4)  | | DIG2 (6)  | DIG3 (12) |
| SEG_D (23) | D (1)  | | DIG3 (7)  | DIG4 (9)  |
| SEG_E (21) | E (3)  | | DIG4 (3)  | DIG5 (8)  |
| SEG_F (15) | F (10) | | DIG5 (10) | DIG6 (5)  |
| SEG_G (17) | G (6)  | | DIG6, DIG7 | *no-connect* (only 6 digits) |
| SEG_DP (22)| DP (2) | |         |             |

This matches the 6-digit display datasheet pinout (segments A=11, B=7, C=4, D=1,
E=3, F=10, G=6, DP=2; digits DIG1..6 = 14,13,12,9,8,5).

## Supporting components added

| Ref | Value | Purpose |
|-----|-------|---------|
| R1, R2 | 33 kΩ | MAX7219 ISET → V+ segment-current set (**required**) |
| R3–R8 | 10 kΩ | pull-ups for the six EXIO buttons (TCA9554 inputs have no internal pull-up) |
| R9–R14 | 10 kΩ | pull-ups on encoder A / B / push lines |
| C1, C3 | 10 µF | MAX7219 V+ bulk decoupling |
| C2, C4 | 100 nF | MAX7219 V+ HF decoupling |
| C5 | 100 nF | ESP32 3V3 decoupling |
| C6 | 100 µF | +5V input bulk cap at J1 |
| C7–C12 | 100 nF | one per SK6812 (VDD–VSS) |
| C13–C18 | 100 nF | RC debounce cap on each encoder A / B / push line |
| J1 | 2-pin screw terminal | +5V / GND power input |

**Encoder debounce:** each encoder A, B and push line has a 10 kΩ pull-up to +3V3
(R9–R14) and a 100 nF cap to GND (C13–C18), forming a ~1 ms RC low-pass that cleans
contact bounce in hardware. Buttons are debounced in firmware.

**Through-hole footprints (this iteration).** All added passives use THT footprints for
easy hand-assembly:

| Part | Footprint |
|------|-----------|
| R1–R14 | `Resistor_THT:R_Axial_DIN0207_L6.3mm_D2.5mm_P7.62mm_Horizontal` (1/4 W axial) |
| 100 nF (C2, C4, C5, C7–C18) | `Capacitor_THT:C_Disc_D5.0mm_W2.5mm_P5.00mm` (ceramic disc) |
| 10 µF (C1, C3) | `Capacitor_THT:CP_Radial_D5.0mm_P2.50mm` (electrolytic) |
| 100 µF (C6) | `Capacitor_THT:CP_Radial_D6.3mm_P2.50mm` (electrolytic) |
| J1 | `TerminalBlock:TerminalBlock_bornier-2_P5.08mm` |

The 10 µF / 100 µF caps (C1, C3, C6) are electrolytic and **polarized** — the terminal on
the `+5V` net is the **+** side. The SK6812 RGB LEDs remain SMD (no THT part exists).

## Design notes / to verify before fab

- **3.3 V logic vs 5 V rails.** MAX7219 V+ and SK6812 VDD are at 5 V while the
  ESP32 drives 3.3 V logic. This usually works but is marginally out of MAX7219 VIH
  spec; for robustness either run the MAX7219 / first SK6812 at ~3.3–4.3 V, or add a
  level shifter on DIN/CLK/LOAD and the LED data line.
- **USB vs J1.** The original design ties +5V to the ESP32 Vbus pin; this schematic
  also feeds Vsys from J1. Do not power from USB and the J1 5 V terminal
  simultaneously unless the two 5 V sources are isolated.
- **Encoder pinout.** EC12E2430803 is wired A/B = quadrature, C = common (GND),
  D/E = push switch. Confirm A/C pin identity against the specific part's datasheet.

## How this was wired & verification status

Connectivity was added as KiCad **global labels placed on each pin's connection
point** (labels of the same name form a net; they merge with the existing `+5V`/`GND`
power symbols). The pin-coordinate transform was calibrated against the wires already
in the file, and a connectivity checker resolved every pin to its net:

- 0 floating pins among wired nets, no shorts, S-expression parentheses balanced.
- Existing partial wiring (SK6812 VDD/VSS rails, GPIO21 → D1.DIN) was preserved.

`kicad-cli` was **not** available in the authoring environment, so the following
must still be run once in KiCad before manufacturing:

1. **ERC** — expect only "unconnected" warnings on the intentionally-spare ESP32 pins.
2. **Board layout / routing** of the newly-added passives (they are placed in a free
   area of the schematic; the PCB copper was intentionally left untouched).
3. **DRC** and **gerber regeneration** into `radiopanelprod/`.
