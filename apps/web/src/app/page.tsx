import { Feature } from '@/components/Feature';

const FEATURES = [
  {
    title: 'Lịch trình theo ngày',
    body: 'Nhập ngày đi, khung giờ và ngân sách. Agent chọn điểm từ 150 địa điểm đã được kiểm tra giờ mở cửa và giá.',
  },
  {
    title: 'Chi tiêu cho cả nhóm',
    body: 'Đặt tổng tiền nhóm và trần chi riêng từng người. Kế hoạch không bao giờ vượt mức của người ít tiền nhất.',
  },
  {
    title: 'Kiểm tra bằng code',
    body: 'Giờ mở cửa, đường đi xe máy, ngân sách và dự báo mưa được validator kiểm tra trước khi lịch được lưu.',
  },
  {
    title: 'Sổ địa điểm & review',
    body: 'Lưu điểm muốn đi, check-in nơi đã đến, review và chia sẻ khoảnh khắc với bạn bè.',
  },
];

export default function Home() {
  return (
    <main>
      <section className="hero">
        <p className="eyebrow">TP.HCM · cuối tuần · nhóm bạn</p>
        <h1>Lên lịch đi chơi Sài Gòn trong vài phút, không phải cả buổi tối.</h1>
        <p className="lead">
          AI Travel Planner dựng lịch trình và kế hoạch chi tiêu theo ngày cho bạn hoặc cả nhóm.
          Không đến nơi đang đóng cửa, không chạy vòng giữa các quận, không tiêu lố ngân sách.
        </p>
        <div className="actions">
          <a className="btn primary" href="#">
            Sắp ra mắt
          </a>
          <a className="btn" href="https://github.com/DuyITLOR/ai-travel-planner">
            Xem trên GitHub
          </a>
        </div>
      </section>

      <section className="grid" aria-label="Tính năng">
        {FEATURES.map((f) => (
          <Feature key={f.title} title={f.title}>
            {f.body}
          </Feature>
        ))}
      </section>

      <footer className="muted">
        Đồ án CSC13114 · Lê Nhựt Duy · Nguyễn Thành Dâng · Trương Thành Đạt
      </footer>
    </main>
  );
}
