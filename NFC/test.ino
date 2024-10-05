
#include <Arduino.h>
#include <MFRC522v2.h>
#include <MFRC522DriverSPI.h>
#include <Wire.h>
#include <MFRC522DriverPinSimple.h>
#include <MFRC522Debug.h>

MFRC522DriverPinSimple ss_pin(10);

MFRC522DriverSPI driver{ss_pin};
MFRC522 mfrc522{driver};

void setup()
{
	Serial.begin(9600);
	while (!Serial)
		;
	mfrc522.PCD_Init();										// Init reader
	MFRC522Debug::PCD_DumpVersionToSerial(mfrc522, Serial); // prints version of nfc reader
}

void loop()
{

	// Looks for new card
	if (!mfrc522.PICC_IsNewCardPresent())
	{
		return;
	}

	// When new card is present, read its data
	if (!mfrc522.PICC_ReadCardSerial())
	{
		return;
	}

	// Print all card information to serial
	MFRC522Debug::PICC_DumpToSerial(mfrc522, Serial, &(mfrc522.uid));
}
