### Base localization / generic strings
### Messages should only go into this file if they are widely used,
### or are particularly important to localize.

## Terms

-service-name = Wikijump

## Special

# Shows whenever a message is still being loaded
message-loading = Đang tải...

goto-home = Về trang chủ

goto-service = Về { -service-name }

base-title = { $title } | { -service-name }

navigated-to = Đã chuyển hướng tới { $path }

## Generic

about = Thông tin
account = Tài khoản
applications = Ứng dụng
avatar = Ảnh đại diện
breadcrumbs = Breadcrumb
change = Thay đổi
clear = xóa
close = Đóng
dashboard = Tổng quan
docs = Tài liệu
download = Tải xuống
edit = Chỉnh sửa
editor = Trình chỉnh sửa
footer = Chân Trang
general = Chung
header = Đầu Trang
help = Trợ giúp
inbox = Hộp thư
invitations = Lời mời
license = Giấy phép
load = Tải
main-content = Nội Dung Chính
messages = Tin nhắn
navigation = Điều hướng
notifications = Thông báo
preview = Xem trước
privacy = Riêng tư
profile = Hồ sơ
publish = Xuất bản
reveal-sidebar = Mở Thanh Cạnh
save = Lưu
security = Bảo mật
send = Gửi
sent = Đã gửi
settings = Cài đặt
sidebar = Thanh cạnh
tags = Tag
terms = Điều khoản
upload = Tải lên

search = Search
  .placeholder = Tìm...

## Generic Authentication

login = Login
  .toast = Bạn đã được đăng nhập.

logout = Logout
  .toast = Bạn đã được đăng xuất.

register = Register
  .toast = Bạn đã được đăng ký.

specifier = Email or Username
  .placeholder = Nhập email hoặc tên người dùng...

username = Username
  .placeholder = Nhập tên người dùng...
  .info = Bạn sẽ có thể thay đổi thông tin này sau.

email = Email
  .placeholder = Nhập địa chỉ email...
  .info = Địa chỉ email của bạn sẽ là riêng tư.

password = Mật khẩu
  .placeholder = Nhập mật khẩu...

confirm-password = Xác Nhận Mật Khẩu

forgot-password = Quên Mật Khẩu
  .question = Bạn đã quên mật khẩu>

reset-password = Đặt Lại Mật Khẩu

remember-me = Giữ đăng nhập

create-account = Tạo Tài Khoản

field-required = Mục này là bắt buộc

characters-left = { $count ->
  [1] 1 ký tự còn lại
  *[other] { $count } ký tự còn lại
}

hold-to-show-password = Giữ để hiện mật khẩu

## Errors

error-404 =
  .generic = Không thể tìm thấy tài nguyên này.
  .page = Không thể tìm thấy trang này.
  .user = Không thể tìm thấy người dùng này.

error-form =
  .missing-fields = Hãy điền vào các mục bắt buộc.
  .password-mismatch = Mật khẩu không khớp.

error-api =
  .GENERIC = Đã xảy ra lỗi với yêu cầu truy cập của bạn.
  .INTERNAL = Một lỗi máy chủ nội bộ đã xảy ra. Hãy thử lại sau.
  .NO_CONNECTION = Bạn không được kết nối với Internet.
  .BAD_SYNTAX = Yêu cầu này không thể được phân tích bởi máy chủ.
  .FORBIDDEN = Bạn không có quyền thực hiện hành động này.
  .NOT_FOUND = Không thể tìm thấy tài nguyên này.
  .CONFLICT = Tài nguyên bạn yêu cầu đang có xung đột với tài nguyên khác.

  .ACCOUNT_ALREADY_VERIFIED = Tài khoản này đã được xác minh trước đó.
  .ACCOUNT_NO_EMAIL = Tài khoản này không có địa chỉ email.
  .ALREADY_LOGGED_IN = Bạn đã được đăng nhập trước đó.
  .FAILED_TO_UPDATE_PROFILE = Không thể cập nhật hồ sơ.
  .INVALID_AVATAR = Tệp được tải lên không phải là một hình ảnh hợp lệ.
  .INVALID_EMAIL = Địa chỉ email không hợp lệ.
  .INVALID_LANGUAGE_CODE = Mã ngôn ngữ không hợp lệ.
  .INVALID_PASSWORD = Mật khẩu sai.
  .INVALID_SESSION = Phiên đăng nhập của bạn đã hết hạn. Hãy đăng nhập lại.
  .INVALID_SPECIFIER = Email hoặc tên người dùng sai.
  .INVALID_USERNAME = Tên người dùng không hợp lệ.
  .LOGIN_FAILED = Không thể đăng nhập. Hãy kiểm tra thông tin của bạn.
  .NOT_LOGGED_IN = Bạn chưa đăng nhập.
  .UNKNOWN_EMAIL = Không có tài khoản nào với địa chỉ email đó.
  .UNKNOWN_USER = Không có tài khoản nào với tên người dùng đó.
  .WRONG_PASSWORD = Sai mật khẩu.

error-418 = Tôi là một ấm trà
