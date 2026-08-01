# Learsim Radio Panel — PCB

KiCad project (`learsim pcbs.kicad_*`) for the Learsim flight-sim **COM/NAV radio
panel**: an ESP32-C6 drives two MAX7219 LED drivers that multiplex two 6-digit
7-segment displays, plus two rotary encoders, six mode buttons with per-button
SK6812 RGB backlights, two indicator LEDs and an SPDT toggle.

## Block diagram

```
         USB-C 5V ── Vbus ─┬─ ESP32-C6 (Vsys)
                           ├─ MAX7219 U2/U3  V+
                           └─ SK6812 D1..D6  VDD
 ESP32-C6 (U1)
   GPIO6  ── SPI DIN ─▶ U2.DIN ─▶ U2.DOUT ─▶ U3.DIN     (daisy-chain)
   GPIO7  ── SPI CLK ─▶ U2.CLK, U3.CLK
   GPIO4  ── SPI LOAD▶ U2.LOAD, U3.LOAD
   GPIO21 ── LED DATA▶ D5 ▶ D1 ▶ D2 ▶ D3 ▶ D4 ▶ D6      (WS2812-style chain)
   GPIO15/16/5  ── Encoder SW1 (MHz)  A / B / push
   GPIO18/19/20 ── Encoder SW2 (KHz)  A / B / push
   GPIO2/0/1/14 ── COM1 COM2 NAV1 Switch1 buttons
   EXIO2 / EXIO3 (TCA9554) ── NAV2 / ADF1 buttons
   GPIO3 / GPIO17 ── COM1 / COM2 indicator LEDs (D7 / D8)
   EXIO6 / EXIO7 (TCA9554) ── SW3 toggle throws A / B

   U2 ─▶ DS2 (6-digit display)      U3 ─▶ DS1 (6-digit display)
```

## ESP32-C6 GPIO map

| Function            | ESP32-C6 pin   | Net       |
|---------------------|----------------|-----------|
| MAX7219 SPI DIN     | GPIO6          | SPI_DIN   |
| MAX7219 SPI CLK     | GPIO7          | SPI_CLK   |
| MAX7219 SPI LOAD/CS | GPIO4          | SPI_LOAD  |
| SK6812 data out     | GPIO21         | LED_DIN   |
| Encoder MHz A / B   | GPIO15 / GPIO16 (TX) | ENC1_A/B |
| Encoder MHz push    | GPIO5          | ENC1_SW   |
| Encoder KHz A / B   | GPIO18 / GPIO19 | ENC2_A/B |
| Encoder KHz push    | GPIO20         | ENC2_SW   |
| COM1 button         | GPIO2          | BTN_COM1  |
| COM2 button         | GPIO0          | BTN_COM2  |
| NAV1 button         | GPIO1          | BTN_NAV1  |
| NAV2 button         | EXIO2 (TCA9554)| BTN_NAV2  |
| ADF1 button         | EXIO3 (TCA9554)| BTN_ADF1  |
| Switch1 button      | GPIO14         | BTN_Switch1 |
| COM1 indicator LED  | GPIO3          | COM1_LED  |
| COM2 indicator LED  | GPIO17 (RX)    | COM2_LED  |
| SW3 toggle throw 1  | EXIO6 (TCA9554)| SW3_A     |
| SW3 toggle throw 2  | EXIO7 (TCA9554)| SW3_B     |
| Power in            | USB-C (Vbus)   | +5V       |
| Logic rail          | 3V3            | +3V3      |

Spare pins for expansion: **GPIO8, GPIO9, EXIO1, EXIO4, EXIO5**. Note GPIO8/GPIO9
are ESP32-C6 strapping/boot pins (GPIO9 low at reset selects download mode) — only
attach loads there that are guaranteed high-or-floating at reset (GPIO8 also drives
the module's onboard RGB LED). **GPIO22/GPIO23 are NOT spare**: they are the I2C
bus (SDA/SCL) to the module's onboard TCA9554 expander that reads the NAV2/ADF1
buttons and SW3 — leave them unconnected; only I2C slaves may ever attach there.

**Encoder MHz B is on the TX pin (GPIO16) and the COM2 LED on RX (GPIO17)**, so
the UART0 serial console is not available; use the ESP32-C6's native USB for
flashing/logs. The ROM bootloader drives TX briefly at reset — harmless for the
encoder, but expect a short glitch on ENC1_B during boot.

**SW3 (SPDT toggle, 2MD1T1B1M2QES-5).** Common pole (pin 2) → GND; each throw read
with a 10 kΩ pull-up to +3V3 (pin 1 → EXIO6 via R15, pin 3 → EXIO7 via R16). A
throw reads LOW when selected and HIGH otherwise, so firmware can distinguish both
positions (and a centre-OFF state if the part has one). SW3 sits on the TCA9554
expander (EXIO6/EXIO7) specifically to keep it **off the GPIO8/GPIO9 strapping
pins** — an ON-ON toggle always grounds one throw, and GPIO9 held low at reset
would put the chip in download mode and stop the board booting. (ENC1_A remains on
GPIO15, also a strapping pin, but an idling encoder there is low-risk. The second
pole of SW3, pins 4–6, is unused.)

## SK6812 backlight chain

The RGB LEDs are reverse-mount SK6812MINI-E on the back of the board, shining
through cutouts under each Cherry-MX button. The data chain order follows board
position, **not** reference-designator order:

```
GPIO21 ▶ D5 (Switch1) ▶ D1 (COM1) ▶ D2 (COM2) ▶ D3 (NAV1) ▶ D4 (NAV2) ▶ D6 (ADF1)
```

Firmware must use this index order (0 = Switch1, 1 = COM1, … 5 = ADF1).

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
E=3, F=10, G=6, DP=2; digits DIG1..6 = 14,13,12,9,8,5). Set the MAX7219 scan
limit to 6 digits in firmware.

## Supporting components

| Ref | Value | Purpose |
|-----|-------|---------|
| R1, R2 | 33 kΩ | MAX7219 ISET → V+ segment-current set (**required**) |
| R3–R8 | 10 kΩ | pull-ups for the six buttons (GPIO2/0/1/23/22/14) |
| R9–R14 | 10 kΩ | pull-ups on encoder A / B / push lines |
| R15, R16 | 10 kΩ | pull-ups on SW3 throws (EXIO6/EXIO7) |
| R17 | 150 Ω | COM1 indicator LED (D7) series resistor |
| R18 | 470 Ω | COM2 indicator LED (D8) series resistor — different value because D7/D8 are different LED colours |
| D7, D8 | 3 mm THT LED | COM1 / COM2 indicator LEDs |
| C1, C3 | 10 µF | MAX7219 V+ bulk decoupling |
| C2, C4 | 100 nF | MAX7219 V+ HF decoupling |
| C5 | 100 nF | ESP32 3V3 decoupling |
| C6 | 220 µF | +5V rail bulk reservoir |
| C7–C12 | 100 nF | one per SK6812 (VDD–VSS) |
| C13–C18 | 100 nF | RC debounce cap on each encoder A / B / push line |

**Encoder debounce:** each encoder A, B and push line has a 10 kΩ pull-up to +3V3
(R9–R14) and a 100 nF cap to GND (C13–C18), forming a ~1 ms RC low-pass that cleans
contact bounce in hardware. Buttons are debounced in firmware.

**Through-hole footprints (this iteration).** All added passives use THT footprints for
easy hand-assembly:

| Part | Footprint |
|------|-----------|
| R1–R18 | `Resistor_THT:R_Axial_DIN0207_L6.3mm_D2.5mm_P7.62mm_Horizontal` (1/4 W axial) |
| 100 nF (C2, C4, C5, C7–C18) | `Capacitor_THT:C_Disc_D5.0mm_W2.5mm_P5.00mm` (ceramic disc) |
| 10 µF (C1, C3) | `Capacitor_THT:CP_Radial_D5.0mm_P2.50mm` (electrolytic) |
| 220 µF (C6) | `Capacitor_THT:CP_Radial_D6.3mm_P2.50mm` (electrolytic) |
| D7, D8 | `LED_THT:LED_D3.0mm` |

The 10 µF / 220 µF caps (C1, C3, C6) are electrolytic and **polarized** — the terminal on
the `+5V` net is the **+** side. The SK6812 RGB LEDs remain SMD (no THT part exists).

## Design notes / to verify before fab

- **PCB reroute pending for SW3.** The schematic moves SW3_A/SW3_B from GPIO8/GPIO9
  (U1 pads 11/12) to EXIO6/EXIO7 (U1 pads 19/20). Run *Update PCB from Schematic*
  and reroute those two tracks, then re-run DRC.
- **3.3 V logic vs 5 V rails.** MAX7219 V+ and SK6812 VDD are at 5 V while the
  ESP32 drives 3.3 V logic. This usually works but is marginally out of MAX7219 VIH
  spec (3.5 V min); for robustness either run the MAX7219s / first SK6812 (D5) at
  ~4.3 V via a diode drop, or add a level shifter on DIN/CLK/LOAD and LED data.
- **+5 V trace width.** The +5 V distribution is currently routed at 0.2 mm while
  the board can draw several hundred mA (6× SK6812 up to ~60 mA each plus two
  MAX7219-driven displays). Widen the 5 V trunk to ≥1 mm or pour it, and consider a
  B.Cu GND pour (this also clears the DRC starved-thermal warnings on U3/SW1).
- **Powered from USB-C.** The board runs entirely off the ESP32-C6 module's USB-C;
  make sure the host/cable can supply the load, and consider capping SK6812
  brightness in firmware if the USB source is current-limited.
- **Encoder pinout.** EC12E2430803 is wired A/B = quadrature, C = common (GND),
  D/E = push switch. Confirm A/C pin identity against the specific part's datasheet.

## Verification status

Checked with KiCad 10 `kicad-cli` (ERC, DRC, netlist export):

- Schematic↔PCB parity clean before the SW3 pin move; SW3 reroute on the PCB is
  the one outstanding layout task (see above).
- ERC: remaining errors are the intentionally-spare pins (GPIO8/GPIO9, EXIO1–5,
  3V3_EN, RUN, MAX7219 DIG6/DIG7, U3 DOUT) plus missing-PWR_FLAG warnings on the
  +5V/+3V3 rails — all benign.
- DRC: 3 starved-thermal errors on the F.Cu GND pour (U3 GND pads, SW1 common);
  connectivity is intact via tracks, and a B.Cu GND pour would clear them.
