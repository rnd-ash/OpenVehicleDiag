# This program converts Daimler CBF Files to the OVD JSON Schema

It can also be used to translate all the German strings in the CBF to a language of your choosing!

---
## Usage

### To just dump the CBF to json (No translation)
```
cbf_parser <INPUT.CBF>
```

### To dump the string table of the CBF (Pre translation)
```
cbf_parser <INPUT.CBF> -dump_strings <OUTPUT.csv>
```

### To translate the Strings in CBF file 
```
python translate_cbf_strings.py <OUTPUT.csv> <LANG>
```
**LANG should be the 2 letter language identifier. EG: 'EN' for English, 'DE' for German, etc...**

### To parse the CBF to json, including the translated strings (Post translation)
```
cbf_parser <INPUT.CBF> -load_strings <OUTPUT.csv_translated>
```

---

## Contributions
Special thanks to [@jglim](https://github.com/jglim) for reverse engineering Caesar for [CaesarSuite](https://github.com/jglim/CaesarSuite). This project is
essentially a smaller version of his code base, and converts the output to the JSON schema OVD supports.
