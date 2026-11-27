import { describe, expect, it } from 'vitest';
import { SignInInput, SignUpInput } from './auth';

describe('SignInInput', () => {
  it('chuẩn hoá email về chữ thường, bỏ khoảng trắng', () => {
    const r = SignInInput.parse({ email: '  Linh@Student.HCMUS.edu.vn ', password: 'x' });
    expect(r.email).toBe('linh@student.hcmus.edu.vn');
  });

  it('báo lỗi khi thiếu email hoặc mật khẩu', () => {
    const r = SignInInput.safeParse({ email: '', password: '' });
    expect(r.success).toBe(false);
    const fields = r.error?.flatten().fieldErrors;
    expect(fields?.email?.[0]).toBe('Nhập email.');
    expect(fields?.password?.[0]).toBe('Nhập mật khẩu.');
  });

  it('báo lỗi khi email sai định dạng', () => {
    const r = SignInInput.safeParse({ email: 'linh@', password: 'x' });
    expect(r.error?.flatten().fieldErrors.email?.[0]).toBe('Email không hợp lệ.');
  });
});

describe('SignUpInput', () => {
  it('từ chối mật khẩu ngắn hơn 8 ký tự', () => {
    const r = SignUpInput.safeParse({ email: 'a@b.vn', password: '1234567' });
    expect(r.success).toBe(false);
  });

  it('chấp nhận mật khẩu đủ 8 ký tự', () => {
    expect(SignUpInput.safeParse({ email: 'a@b.vn', password: '12345678' }).success).toBe(true);
  });
});
