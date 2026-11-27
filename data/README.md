# AI Travel Planner — Bộ dữ liệu địa điểm dùng chung (lớp 2609-AWAD-23KTPM1)

Nhóm product owner (Lê Nhựt Duy, Nguyễn Thành Dâng, Trương Thành Đạt) công bố bộ dữ liệu này cho tất cả các team, theo yêu cầu ở mục 6 của brief.
Ngày kiểm tra: **24/11/2026**. Nếu thấy giờ mở cửa hoặc giá bị sai, vui lòng báo dưới bài đăng *"AI Travel Planner — questions for the product owners"* trong group Facebook của môn (`#AWAD23KTPM1`), kèm `id` của địa điểm.

## Các file

| File | Nội dung | Tool sử dụng |
| --- | --- | --- |
| `places.json` | 150 địa điểm: category, quận, địa chỉ, tọa độ, giờ mở cửa từng ngày trong tuần, giá vé, trong nhà/ngoài trời, mức giá, sở thích, ảnh, nguồn | `search_places` |
| `menu_items.json` | Các món hoặc loại vé, giá là số nguyên VND, liên kết qua `place_id` | `get_menu`, `compute_cost` |
| `travel_time_matrix.json` | Thời gian đi xe máy (phút) và quãng đường (mét) giữa mọi cặp địa điểm | `get_travel_time` |
| `images/` | Ảnh có giấy phép mở (Wikimedia Commons, một ít từ Openverse/Flickr) cho **63/150 địa điểm** (11 ảnh lấy từ trang chính thức, ghi `license: "chưa rõ…"` — cần xin phép hoặc thay trước khi dùng công khai); thông tin credit nằm trong `images[]` của từng địa điểm. Phần lớn quán ăn không có ảnh giấy phép mở: hãy hiển thị ảnh đại diện theo category | trang chi tiết |
| `csv/` | Cùng dữ liệu ở dạng CSV (UTF-8 có BOM, mở được bằng Excel/Google Sheets) để xem nhanh | — |

## Category (12 loại)

| `category` | Tên tiếng Việt | Số lượng |
| --- | --- | --- |
| `museum` | Bảo tàng | 12 |
| `landmark` | Địa danh & công trình | 12 |
| `religious_site` | Chùa, nhà thờ, đền | 12 |
| `park_nature` | Công viên & thiên nhiên | 10 |
| `entertainment` | Vui chơi giải trí | 11 |
| `art_culture` | Nghệ thuật, workshop & show | 9 |
| `shopping_mall` | Trung tâm thương mại | 11 |
| `market` | Chợ | 13 |
| `street_food` | Ăn vặt & quán bình dân | 20 |
| `restaurant` | Nhà hàng | 15 |
| `cafe` | Cà phê & trà | 18 |
| `nightlife` | Bar & về đêm | 7 |

## Các trường trong `places.json`

| Trường | Kiểu | Ghi chú |
| --- | --- | --- |
| `id` | string | `p001`…`p150`, cố định — dùng làm khóa |
| `name`, `name_en` | string | tên chính thức tiếng Việt; tên tiếng Anh hoặc null |
| `category`, `category_vi` | string | xem bảng ở trên |
| `district` | string | **tên quận cũ** (Quận 1, Bình Thạnh, Thủ Đức (Quận 2)…) như mọi người vẫn gọi; địa chỉ đầy đủ nằm ở `address` |
| `lat`, `lon` | number | hệ tọa độ WGS84 |
| `coord_precision` | `"poi"` \| `"street"` | `poi` = đúng tòa nhà/lối vào (lấy từ OSM, trang Foody hoặc bản đồ chính thức, đã kiểm tra chéo bằng reverse geocoding); `street` = chỉ biết tên đường (5 địa điểm, đã đánh dấu để kiểm tra tay) |
| `indoor_outdoor` | `"indoor"` \| `"outdoor"` | validator sẽ chặn các điểm `outdoor` khi khả năng mưa trên 60% |
| `ticket_price_vnd` | integer | giá vé người lớn; 0 = vào cửa miễn phí (kể cả nhà hàng/quán cà phê) |
| `price_level` | 1–4 | mức chi tiêu thường gặp mỗi người: 1 < 50k, 2 = 50–150k, 3 = 150–400k, 4 > 400k VND |
| `typical_duration_min` | integer | thời gian tham quan thường mất (phút) |
| `opening_hours` | object | `mon`…`sun` → danh sách `["HH:MM","HH:MM"]`; `[]` = nghỉ ngày đó; giờ đóng cửa sau nửa đêm ghi quá 24 (`["17:00","26:00"]` = 17:00–02:00); mở cả ngày = `["00:00","24:00"]` |
| `opening_hours_note` | string \| null | ngoại lệ (ngày lễ, giờ bán vé cuối, …) |
| `interests` | string[] | chọn trong: history, culture, art, architecture, religion, nature, photography, food, street_food, cafe, nightlife, shopping, entertainment, local_life, family, music, view |
| `images` | object[] | `file`, `author`, `license`, `license_url`, `page` (phải ghi credit) |
| `sources` | object[] | `url` + phần thông tin nguồn đó cung cấp (`covers`: hours, prices, menu, address, status) |
| `checked_at` | date | ngày đọc nguồn |
| `verification` | object | `status: "web_sourced"`, `confidence: high/medium/low`, `human_checked` |
| `notes` | string \| null | nguồn mâu thuẫn, địa điểm được thay thế, lưu ý khác |

## `menu_items.json`

`{ "item_id": "p001-m1", "place_id": "p001", "name": "Vé người lớn", "kind": "ticket" | "food" | "drink" | "service", "price_vnd": 60000 }`
Giá là số nguyên VND. Khi nguồn chỉ ghi khoảng giá, dữ liệu dùng mức giá phổ biến và có ghi rõ trong `notes` của địa điểm.

## `travel_time_matrix.json`

`place_ids[i]` ứng với hàng/cột `i` của `minutes` và `distance_m`. Ma trận được tính bằng OSRM (profile ô tô trên mạng đường OpenStreetMap) rồi quy đổi sang thời gian đi xe máy:
`minutes = ceil(car_minutes × 1.4 + 3)` — hệ số 1.4 bù cho kẹt xe trong thành phố (tốc độ của OSRM là tốc độ đường trống), 3 phút là thời gian xuất phát và gửi xe. Đường chéo = 0.

## Quyết định của product owner (áp dụng cho mọi team)

1. **Số món trong menu.** Quán ăn, quán cà phê, bar và trung tâm thương mại có 3–8 món có giá. Điểm tham quan (bảo tàng, địa danh, chùa/nhà thờ, công viên, chợ, vui chơi, nghệ thuật) có thể có ít hơn, tối thiểu 1 món, nếu đó là tất cả những gì du khách phải trả — phần lớn là miễn phí (`"Vào cửa"`, 0 VND). Một địa điểm được làm candidate khi có giờ mở cửa và ít nhất một món.
2. **Không đưa vào các show không có giờ cố định.** Các sân khấu chỉ mở vào ngày diễn đã được thay bằng địa điểm có giờ mở cửa cố định, để có thể kiểm tra điều kiện "trong giờ mở cửa".
3. **Dùng tên quận cũ** (trước đợt sắp xếp năm 2025), vì người dùng và các nguồn vẫn gọi như vậy.
4. **Giá chỉ là ước tính để lên kế hoạch,** không phải báo giá. Với các quán chỉ có giá trên app giao đồ ăn (ShopeeFood/GrabFood), giá thường **cao hơn** giá tại quán (có món gần gấp đôi), nên kế hoạch dựa trên các giá này sẽ ước dư, an toàn cho ngân sách; mỗi địa điểm như vậy đều ghi rõ trong `notes`.
5. **Saigon Square được xếp vào `market`,** không phải trung tâm thương mại: đây là khu sạp hàng, không có giá cố định.

## Cách kiểm tra dữ liệu

Mỗi địa điểm được tra cứu trên web vào ngày 24/11/2026: ưu tiên website và fanpage chính thức, sau đó đến app giao đồ ăn, rồi tin tức và cẩm nang du lịch năm 2025–2026. Mọi giờ mở cửa và giá đều lấy từ một nguồn có trong `sources`. Giá trị nào không tìm được nguồn thì bỏ trống chứ không đoán, và địa điểm đã đóng cửa thì được thay thế (xem `notes`). Trường `verification.confidence` cho biết mức độ tin cậy của từng địa điểm: `low` nghĩa là các nguồn mâu thuẫn hoặc đã cũ. Không có dữ liệu nào lấy từ Google Maps / Google Places.

## Ghi nguồn (hiển thị trong sản phẩm của các team)

- Dữ liệu bản đồ và đường đi: © [OpenStreetMap](https://www.openstreetmap.org/copyright) contributors, giấy phép ODbL. Định tuyến: [OSRM](https://project-osrm.org/).
- Ảnh: [Wikimedia Commons](https://commons.wikimedia.org/) — tác giả và giấy phép của từng ảnh nằm trong `images[]`.
- Thời tiết (dùng cho `get_weather`, không có trong thư mục này): [Open-Meteo](https://open-meteo.com/), CC BY 4.0.
