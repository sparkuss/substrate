License: Unlicense

Current Types needed:

{
  "Address": "IndicesLookupSource",
  "LookupSource": "IndicesLookupSource",
  "GameEventType": {
    "_enum": [
      "Default",
      "Hatch"
    ]
  },
  "GameEvent": {
    "id": "H256",
    "begin": "BlockNumber",
    "duration": "u16",
    "event_type": "GameEventType",
    "hashes": "Vec<H256>",
    "value": "u64"
  },
  "MogwaiStruct": {
    "id": "H256",
    "dna": "H256",
    "genesis": "BlockNumber",
    "price": "Balance",
    "gen": "u64"
  },
  "MogwaiBios": {
	"mogwai_id": "Hash",
	"state": "u32",
	"metaxy": "Vec<[u8;16]>",
	"intrinsic": "Balance",
	"level": "u8",
	"phases": "Vec<BlockNumber>",
	"adaptations": "Vec<Hash>"
  }
}