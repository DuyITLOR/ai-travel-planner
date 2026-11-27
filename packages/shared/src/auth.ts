import { z } from 'zod';

/** Độ dài tối thiểu của mật khẩu; khớp `minimum_password_length` trong supabase/config.toml. */
export const PASSWORD_MIN_LENGTH = 8;

const email = z
  .string()
  .trim()
  .min(1, 'Nhập email.')
  .email('Email không hợp lệ.')
  .transform((v) => v.toLowerCase());

/** Form đăng nhập: chỉ kiểm tra có nhập, độ dài để Supabase quyết định. */
export const SignInInput = z.object({
  email,
  password: z.string().min(1, 'Nhập mật khẩu.'),
});
export type SignInInput = z.infer<typeof SignInInput>;

/** Form đăng ký: mật khẩu phải đủ độ dài tối thiểu. */
export const SignUpInput = z.object({
  email,
  password: z
    .string()
    .min(PASSWORD_MIN_LENGTH, `Mật khẩu cần ít nhất ${PASSWORD_MIN_LENGTH} ký tự.`)
    .max(72, 'Mật khẩu tối đa 72 ký tự.'),
});
export type SignUpInput = z.infer<typeof SignUpInput>;
