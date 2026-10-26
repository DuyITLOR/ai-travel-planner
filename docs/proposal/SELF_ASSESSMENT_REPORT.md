# SELF_ASSESSMENT_REPORT — PA#1 Proposal and planning

**Nhóm:** 23127178 Lê Nhựt Duy · 23127334 Nguyễn Thành Dâng · 23127344 Trương Thành Đạt
**Repository:** https://github.com/DuyITLOR/ai-travel-planner
**Tài liệu nộp:** `AI_Travel_Planner_Proposal.pdf` (2 trang A4, 6 mục theo đúng 6 tiêu chí của rubric; bản `.docx` cùng nội dung kèm theo để chỉnh sửa). Bản đầy đủ có sơ đồ luồng Agent, kiến trúc, mô hình dữ liệu và bảo mật nằm trong repo tại `docs/proposal-full.docx`.

| # | Tiêu chí | Tối đa | Tự chấm | Bằng chứng |
|---|---|---|---|---|
| 1 | Problem and users | 20 | 19 | Mục 1 của proposal: người dùng có tên và tình huống cụ thể (Linh, 20 tuổi, sinh viên Cần Thơ, 2 ngày cuối tuần ở TP.HCM cùng 3 bạn, 1 triệu/người, Grab và xe máy); tình huống là trải nghiệm thật của cả ba thành viên năm nhất khi mới lên Sài Gòn; vấn đề nêu trong một câu; đoạn "Hiện tại họ làm thế nào" liệt kê TikTok, Google Maps, fanpage, Notes, Zalo và chỗ hay sai; lý do hỏi ChatGPT không đủ. |
| 2 | The LLM feature, and the cost of it being wrong | 25 | 24 | Mục 2: một tính năng duy nhất (Agent lập lịch trình và chi tiêu theo ngày) mô tả tới mức 7 tool có tên và kiểu trả về, luồng 6 bước với hai điểm người dùng xác nhận, luật validator chạy bằng code. Bảng "Cái giá khi sai" 4 dòng, mỗi dòng ghi ai chịu, mất bao nhiêu (phút, tiền, phần trăm ngày), hoàn tác được không. Đoạn "Làm sao biết nó sai" nêu log, nút báo lỗi, bộ 30 ca và phép so sánh thủ công với Agent; mục tiêu đo được ghi rõ là mục tiêu sẽ đo ở CP4 (tool đúng từ 80%, 0 lịch vượt trần chi sau validator, p90 dưới 60 giây). |
| 3 | Scope: in and out | 15 | 14 | Mục 3: danh sách "Trong học kỳ" với số CP hoàn thành ghi sau từng mục, khớp với bảng kế hoạch mục 4; danh sách "Chủ động không làm" (đặt vé, thanh toán, chat, bảng tin cá nhân hoá, ngoài TP.HCM, chatbot tự do, API địa điểm lúc tạo lịch, Google Places, chỉ đường, tiền Grab). |
| 4 | Plan and ownership | 20 | 18 | Mục 4: bảng 6 checkpoint từ 26/10 đến 25/11 (26/10, 01/11, 07/11, 13/11, 19/11, 25/11), mỗi dòng có việc gắn tên người phụ trách (Đạt, Duy, Dâng) và điều kiện đạt kiểm tra được; đường găng nêu rõ; lịch do nhóm tự ràng buộc để hoàn thành đồ án trong một tháng, nằm trước hạn của môn. Mỗi việc là một issue trên GitHub Projects của repo. |
| 5 | Risks | 10 | 10 | Mục 5: hai rủi ro riêng của dự án (R1 không có giờ mở cửa và giá thật để validator đối chiếu, có số đo từ lần thử 14/10; R2 model 7B gọi tool sai hoặc quá chậm), mỗi rủi ro có việc bắt đầu tuần này kèm tên người làm, ngưỡng số cụ thể và phương án B. |
| 6 | Technology choices | 10 | 10 | Mục 6: bảng 11 lựa chọn, mỗi dòng một câu lý do gắn với dự án và chi phí; model chính Qwen 2.5 7B qua Ollama (0 đồng, máy 8 GB RAM) và dự phòng Gemini 3.5 Flash-Lite với giá $0.30 / $2.50 mỗi 1M token, ước tính $0.02 mỗi lượt và $3 xấu nhất cho học kỳ; 5 link nguồn giá kiểm tra lại ngày 20/10/2026, số liệu khớp trang chính thức (Gemini, Render 750 giờ và ngủ sau 15 phút, Firestore 1 GiB / 50K đọc / 20K ghi mỗi ngày, Cloudinary 25 credit, Open-Meteo 10.000 lần/ngày CC BY 4.0). |
| | **Tổng** | **100** | **95** | |

## What we did not manage

- Các ngưỡng của tính năng LLM (80% tool đúng, 60 giây) là mục tiêu, chưa phải số đo; spike đầu tiên là việc của CP1.
- Chưa có wireframe màn hình Agent và màn hình chi tiêu; sẽ nộp cùng spec IA#1 ở CP2.
- Chi phí mỗi lượt Agent (30k token vào, 4k token ra) là ước tính từ kích thước tool, chưa đo bằng script thật.
- Lịch một tháng là nhóm tự đặt để dồn sức hoàn thành đồ án sớm; nếu CP4 trễ thì CP5 sẽ bị cắt theo thứ tự đã ghi trong bảng điều kiện.
- Repo mới có khung thư mục và README; chưa có commit dữ liệu địa điểm (150 điểm là việc của CP1).

**Tên file zip:** `23127178-23127334-23127344_95.zip`
