/** Đổi lỗi của Supabase Auth sang câu tiếng Việt cho người dùng. Không lộ chi tiết nội bộ. */
export function authErrorMessage(
  error: { code?: string; status?: number } | null | undefined,
): string {
  switch (error?.code) {
    case 'invalid_credentials':
      return 'Email hoặc mật khẩu không đúng.';
    case 'email_not_confirmed':
      return 'Email chưa được xác nhận. Mở thư xác nhận trong hộp thư rồi thử lại.';
    case 'user_already_exists':
    case 'email_exists':
      return 'Email này đã có tài khoản. Hãy đăng nhập.';
    case 'email_address_invalid':
      return 'Không gửi được thư xác nhận tới email này. Dùng một email thật khác.';
    case 'email_address_not_authorized':
      return 'Máy chủ chưa được phép gửi thư tới email này (chưa cấu hình SMTP).';
    case 'weak_password':
      return 'Mật khẩu quá yếu. Thử mật khẩu dài hơn.';
    case 'over_request_rate_limit':
    case 'over_email_send_rate_limit':
      return 'Bạn thử quá nhiều lần. Đợi một lát rồi thử lại.';
    case 'signup_disabled':
      return 'Hiện chưa mở đăng ký tài khoản mới.';
    case 'provider_disabled':
      return 'Đăng nhập bằng Google chưa được bật.';
  }
  if (error?.status === 429) return 'Bạn thử quá nhiều lần. Đợi một lát rồi thử lại.';
  return 'Không đăng nhập được lúc này. Thử lại sau.';
}
