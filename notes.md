## STTM (from main.rs patch_sttm_dict())
The 'Data Dictionary' is failing to tag some fields for some reports:
| Column         | Missing Reports    |
| -------------- | ------------------ |
| flow_direction | INT715A, INT715B   |
| trn            | INT705v2, INT705v3 |
| trn_priority   | INT706v2           |

The following are a list of reports that are implied to be v2 (or higher) by filename, but are referenced in the 'Data Dictionary' as v1 (without version number):
- `INT718v2` 
- `INT656v2`
- `INT657v2`
- `INT653v3`

The field name `market_position` (Data Dictionary) is misspelt for record `INT724` as `market_postition`

Facillity fields for `INT656` have a prepending capital letter:
- `Facility_identifier` (`facility_identifier`)
- `Facility_name` (`facility_name`)