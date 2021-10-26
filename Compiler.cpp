/*
 * EasyGAL.cpp
 *
 *  Created on: May 28, 2020
 *      Author: elias
 */

#include <iostream>
#include <vector>

#include "Shared/Validate.h"
#include "Shared/TableData.h"
#include "Shared/Dependencies/json.hpp"
#include "Shared/Utility.h"
#include "Shared/API.h"

#include "Parser/Parser.h"
#include "Parser/Error.h"

#include "Translator/Translator.hpp"
#include "Translator/Configs.h"

using namespace std;

void compile(string easyGALCode, string outputFileName, string deviceName)
{
	Parser parser = Parser(easyGALCode);
	vector<TableData> tableData = parser.parse();

	Configs::CircuitConfig DeviceType;
	vector<uint32_t> inputPins;
	vector<uint32_t> outputPins;
	initDeviceType(DeviceType, deviceName, inputPins, outputPins);
	validate(tableData, inputPins, outputPins);

	Translator::Process(tableData, DeviceType, outputFileName);

	cout << "compilation successfully, new jedec file was created " << outputFileName << endl;
}

void cli(int argc, char *argv[])
{
	if (argc == 1)
	{
		cerr << "invalid argument count" << endl;
		showHelpMenu();
		exit(1);
	}

	if (strcmp(argv[1], "help") == 0)
	{
		showHelpMenu();
		exit(0);
	}
	else if (strcmp(argv[1], "api") == 0)
	{
		if (argc == 2)
		{
			cerr << "invalid argument count" << endl;
			showHelpMenu();
			exit(1);
		}

		string fileEnding = getFileEnding(argv[2]);
		if (fileEnding == "json")
		{
			checkFileEnding(argv[3], "jedec");
			if (argc != 5)
			{
				cerr << "invalid argument count" << endl;
				showHelpMenu();
				exit(1);
			}
			api::tableData2jedec(argv[2], argv[3], argv[4]);
		}
		else if (fileEnding == "txt")
		{
			checkFileEnding(argv[3], "json");
			if (argc != 4 && argc != 5)
			{
				cerr << "invalid argument count" << endl;
				showHelpMenu();
				exit(1);
			}
			api::code2TableData(argv[2], argv[3], argc == 5 ? argv[4] : "");
		}
		else
		{
			cerr << "invalid file extention " + string(argv[2]) << endl;
			showHelpMenu();
			exit(1);
		}
	}
	else
	{
		if (argc == 4)
		{
			checkFileEnding(argv[1], "txt");
			checkFileEnding(argv[2], "jedec");
			compile(argv[1], argv[2], argv[3]);
		}
		else
		{
			cerr << "invalid argument count" << endl;
			showHelpMenu();
			exit(1);
		}
	}
}

uint8_t ConvertBoolArrayToByte(vector<bool> source)
{
	uint8_t result = 0;
	for (int i = 0; i < 8; i++)
		if (source[i])
			result |= (uint8_t)(1 << (7 - i));
	return result;
}

void printTableData(TableData tableData)
{
	printf("TableData { output_pin: %d, enable_flip_flop: %s, input_pins: [", tableData.m_OutputPin, tableData.m_EnableFlipFlop ? "true" : "false");
	for (uint32_t pin : tableData.m_InputPins)
		printf("%d, ", pin);
	printf("], table: [");
	for (bool b : tableData.m_Table)
		printf("%s, ", b ? "true" : "false");
	printf("]}\n");
}

void printNewTableData(TableData tableData)
{
	printf("TableData::new(vec![ ");
	for (uint32_t pin : tableData.m_InputPins)
		printf("%d, ", pin);
	printf("], %d, vec![", tableData.m_OutputPin);
	for (bool b : tableData.m_Table)
		printf("%s, ", b ? "true" : "false");
	printf("], %s)\n", tableData.m_EnableFlipFlop ? "true" : "false");
}

void printFusesBytes(vector<bool> Fuses)
{
	int index = 0;
	vector<bool> byte;

	for (int i = 0; i < Fuses.size() / 8; i++)
	{
		for (int j = 0; j < 8; j++)
			byte.push_back(Fuses.at(i * 8 + j));

		printf("0x%02hhX, ", ConvertBoolArrayToByte(byte));
		byte.clear();
	}
}

int main(int argc, char *argv[])
{

	Configs::CircuitConfig Config;
	vector<uint32_t> inputPins;
	vector<uint32_t> outputPins;
	initDeviceType(Config, "g22v10", inputPins, outputPins);

	vector<json> json_vec{R"(
    {
      "dff": true,
      "inputPins": [10, 11],
      "outputPin": 23,
      "table": [false, false, true, false]
    }
)"_json,
						  R"({
      "dff": false,
      "inputPins": [10, 11],
      "outputPin": 17,
      "table": [false, false, false, true]
    })"_json,
						  R"({
      "dff": false,
      "inputPins": [10, 11],
      "outputPin": 19,
      "table": [false, true, true, false]
    })"_json,
						  R"({
      "dff": false,
      "inputPins": [10, 11],
      "outputPin": 18,
      "table": [false, true, true, true]
    })"_json,
						  R"({
      "dff": true,
      "inputPins": [3, 2],
      "outputPin": 23,
      "table": [true, true, false, true]
    })"_json,
						  R"({
      "dff": true,
      "inputPins": [3, 2],
      "outputPin": 23,
      "table": [false, true, true, false]
    })"_json};
	vector<TableData> TruthTables = api::parseTableDataArray(json_vec);

	printf("\n\nvec![");

	for (TableData TruthTable : TruthTables)
	{
		printNewTableData(TruthTable);
		printf(", ");
	}
	printf("];\n\n");

	vector<DNF::Expression> Expressions;
	if (!DNF::Build(TruthTables, Expressions, &Config))
	{
		ERROR("%s", "couldn't build all DNF expressions");
		return false;
	}

	vector<bool> Fuses;

	Configs::CircuitConfig *ConfigPtr = std::addressof(Config);

	uint32_t iRowLength = Fuses::GetRowLength(ConfigPtr);
	uint32_t iNumRows = Fuses::Output::MaximumTerms(Expressions[0].m_OutputPin, ConfigPtr) + 1;

	if (!Fuses::BuildFromExpression(Expressions[0], iNumRows, iRowLength, Fuses, ConfigPtr))
	{
		ERROR("%s", "couldn't generate all fuses for given expressions");
		return false;
	}

	printf("\n\n");
	DNF::printNewExpression(Expressions[0]);
	printf("\n\n");
	printf("vec![");
	printFusesBytes(Fuses);
	printf("]");
}