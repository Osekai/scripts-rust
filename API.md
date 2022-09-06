root: /rankings/api/upload/scripts-rust/


POST up_medals.php
data example

```json
[{
	"medalid": 1,
	"name": "500 Combo",
	"link": "https://assets.ppy.sh/medals/web/osu-combo-500.png",
	"description": "500 big ones! You're moving up in the world!",
	"restriction": "osu",
	"grouping": "Skill",
	"instructions": "aiming for a combo of 500 or higher on any beatmap",
	"ordering": 0
}]
```

... etc. should be an array ob those objects, one for each meadl


POST up_medals_rarity.php
data example

```json
[{
	"medalid": 1,
	"frequency": 83.5266
}, 
{
	"medalid": 2,
	"frequency": 72.4523
}]
```


POST up_ranking.php
data example
```json
[
    {
        "id": "4687701",
        "name": "xxluizxx47",
        "total_pp": "31073",
        "stdev_pp": "22352",
        "standard_pp": "14166",
        "taiko_pp": "6145",
        "ctb_pp": "4359",
        "mania_pp": "6404",
        "medal_count": "268",
        "rarest_medal": "192",
        "country_code": "BR",
        "standard_global": "154",
        "taiko_global": "1609",
        "ctb_global": "2427",
        "mania_global": "8145",
        "badge_count": "1",
        "ranked_maps": "1",
        "loved_maps": "0",
        "subscribers": "104",
        "replays_watched": "51605",
        "avatar_url": "https://a.ppy.sh/4687701?1646537240.jpeg"
    },
    {
        "id": "10238680",
        "name": "chromb",
        "total_pp": "20913",
        "stdev_pp": "17585",
        "standard_pp": "6360",
        "taiko_pp": "6862",
        "ctb_pp": "3331",
        "mania_pp": "4360",
        "medal_count": "256",
        "rarest_medal": "217",
        "country_code": "GB",
        "standard_global": "24069",
        "taiko_global": "1053",
        "ctb_global": "3680",
        "mania_global": "24188",
        "badge_count": "1",
        "ranked_maps": "0",
        "loved_maps": "0",
        "subscribers": "7",
        "replays_watched": "193",
        "avatar_url": "https://a.ppy.sh/10238680?1662021805.png"
    }
]
```
should be literally every user


GET finish.php
no data, hit this when done
