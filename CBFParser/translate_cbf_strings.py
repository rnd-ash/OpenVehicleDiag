import os
import sys
import re
from googletrans import Translator

translator = Translator(service_urls=["translate.google.com", "translate.google.co.kr",
                      "translate.google.at", "translate.google.de",
                      "translate.google.ru", "translate.google.ch",
                      "translate.google.fr", "translate.google.es"])


TRANSLATE_BATCH_SIZE = 100

#
# USAGE: translate_cbf_strings.py <INPUT.csv>
# (Strings will be translated in place)

stringTable = dict()
lang = sys.argv[2]
tmp = open(sys.argv[1], 'r').read().split("\"\"\"\"\n")
lines=[]
for e in tmp:
    split = e.split(",\"\"\"\"")
    try:
        lines += tuple((int(split[0]), split[1]))
    except Exception:
        continue
lineCount = len(lines)

linesCleaned = []
print(lines)
for (idx,line) in lines:
    try:
        # If this succeeds then its a hex string so ignore it!
        bytes.fromhex(line.replace(" ",""))
    except Exception:
        try:
            # Its an Int so ignore it
            int(line.replace(" ",""))
        except Exception:
            if len(line) > 2 and not line.isalpha():
                linesCleaned.append((idx, line))


# Now flip it so we have string paired to every index in the CBF where the string is found, reduces the number of strings to parse
for (idx, string) in linesCleaned:
    if string in stringTable:
        stringTable[string].append(idx)
    else:
        stringTable[string] = [idx]

symbolList = []

for s in stringTable.keys():
    # CBF uses a combination of :, _, -, ' ' to separate strings
    for res in re.split('(_|:| |-|\.)', s):
        if res not in symbolList:
            symbolList.append(res)

toTranslate = []

for s in symbolList:
    if len(s) > 1:
        toTranslate.append(s)

batches = [toTranslate[i:i + TRANSLATE_BATCH_SIZE] for i in range(0, len(toTranslate), TRANSLATE_BATCH_SIZE)]


# Reduce further by deleting strings that are just numbers (For some reason they exist)

print(batches)
print("String grouping complete. {} entries combined to {} strings {} symbols".format(len(lines), len(stringTable), len(toTranslate)))
print("Using {} google translate API calls".format(len(batches)))

symbolLookupTable=dict()

for b in batches:
    ret = ""
    print("Translating")
    for s in b:
        ret += s + "\n"
    to_translate = ret.encode('utf8').decode('utf8') + '\n'
    result = translator.translate(to_translate, src="DE", dest=lang).text.split("\n")
    for x in range(len(b)):
        symbolLookupTable[b[x]] = result[x]

# Now translation is completed, stitch everything back in the reverse order!

print("Stitching strings back together")

completedStrings=dict()

for s in stringTable.keys():
    # CBF uses a combination of :, _, -, ' ' to separate strings
    symbols = re.split('(_|:| |-|\.)', s)

    for x in range(len(symbols)):
        if symbols[x] in symbolLookupTable:
            symbols[x] = symbolLookupTable[symbols[x]]
        # Replace any found text with translated text


    newString = ''.join(symbols)
    completedStrings[newString] = stringTable[s]
print(completedStrings)

completedLines=[None]*lineCount
# Now replace the content in the lines array with the translated strings
for string in completedStrings.keys():
    for idx in completedStrings[string]:
        completedLines[idx] = (idx, string)

for (idx,line) in lines:
    if completedLines[idx] == None:
        completedLines[idx] = (idx, line)

with open(sys.argv[1] + "_translated", 'w') as f:
    for (idx, line) in completedLines:
        f.write("{},\"{}\"\n".format(idx, line))
        