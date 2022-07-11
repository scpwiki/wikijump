### Base localization / generic strings
### Messages should only go into this file if they are widely used,
### or are particularly important to localize.

## Terms

-service-name = Wikijump

## Special

# Shows whenever a message is still being loaded
message-loading = 불러오는 중...

goto-home = 홈페이지로 가기

goto-service = { -service-name }(으)로 가기

base-title = { $title } | { -service-name }

navigated-to = { $path }(으)로 이동함

## Generic

about = 정보
account = 계정
applications = 애플리케이션
avatar = 아바타
breadcrumbs = 이동 경로
change = 바꾸기
clear = 지우기
close = 닫기
dashboard = 대시보드
docs = 문서
download = 다운로드
edit = 편집
editor = 편집기
footer = 페이지 푸터
general = 일반
header = 페이지 헤더
help = 도움말
inbox = 사서함
invitations = 초대
license = 라이선스
load = 불러오기
main-content = 주 콘텐츠
messages = 메시지
navigation = 탐색
notifications = 알림
preview = 미리보기
privacy = 개인 정보
profile = 프로필
publish = 게시
reveal-sidebar = 사이드바 보이기
save = 저장
security = 보안
send = 보내기
sent = 보냄
settings = 설정
sidebar = 사이드바
tags = 태그
terms = 이용 약관
upload = 업로드

search = 검색
  .placeholder = 검색...

## Generic Authentication

login = 로그인
  .toast = 로그인되었습니다.

logout = 로그아웃
  .toast = 로그아웃되었습니다.

register = 가입하기
  .toast = 가입되었습니다.

specifier = 이메일 또는 사용자명
  .placeholder = 이메일 또는 사용자명을 입력하세요.

username = 사용자명
  .placeholder = 사용자명을 입력하세요.
  .info = 나중에 바꿀 수 있습니다.

email = 이메일
  .placeholder = 이메일 주소를 입력하세요.
  .info = 이메일 주소는 공개되지 않습니다.

password = 비밀번호
  .placeholder = 비밀번호를 입력하세요.

confirm-password = 비밀번호 확인

forgot-password = 비밀번호를 잊음
  .question = 비밀번호를 잊으셨나요?

reset-password = 비밀번호 재설정하기

remember-me = 로그인 상태 유지하기

create-account = 계정 만들기

field-required = 필수 입력란입니다.

characters-left = { $count }자 남음

hold-to-show-password = 비밀번호를 보려면 길게 누르기

## Errors

error-404 =
  .generic = 요청한 리소스를 찾을 수 없습니다.
  .page = 요청한 페이지를 찾을 수 없습니다.
  .user = 요청한 사용자를 찾을 수 없습니다.

error-form =
  .missing-fields = 필수 입력란을 모두 채워주세요.
  .password-mismatch = 비밀번호가 같지 않습니다.

error-api =
  .GENERIC = 요청이 잘못되었습니다.
  .INTERNAL = 내부 서버 오류가 발생했습니다. 나중에 다시 시도해주세요.
  .NO_CONNECTION = 인터넷에 연결되지 않았습니다.
  .BAD_SYNTAX = 서버가 요청을 처리할 수 없습니다.
  .FORBIDDEN = 이 동작을 수행할 권한이 없습니다.
  .NOT_FOUND = 요청한 리소스를 찾을 수 없습니다.
  .CONFLICT = 요청한 리소스가 다른 리소스와 충돌합니다.

  .ACCOUNT_ALREADY_VERIFIED = 이 계정은 이미 인증되었습니다.
  .ACCOUNT_NO_EMAIL = 이 계정에는 등록된 이메일 주소가 없습니다.
  .ALREADY_LOGGED_IN = 이미 로그인되었습니다.
  .FAILED_TO_UPDATE_PROFILE = 프로필을 갱신할 수 없습니다.
  .INVALID_AVATAR = 업로드된 파일은 올바른 이미지가 아닙니다.
  .INVALID_EMAIL = 이메일 주소가 올바르지 않습니다.
  .INVALID_LANGUAGE_CODE = 언어 코드가 올바르지 않습니다.
  .INVALID_PASSWORD = 비밀번호가 올바르지 않습니다.
  .INVALID_SESSION = 세션이 만료되었습니다. 다시 로그인해주세요.
  .INVALID_SPECIFIER = 이메일 또는 사용자명이 올바르지 않습니다.
  .INVALID_USERNAME = 사용자명이 올바르지 않습니다. 
  .LOGIN_FAILED = 로그인할 수 없습니다. 자격 증명을 확인해주세요.
  .NOT_LOGGED_IN = 로그인되지 않았습니다.
  .UNKNOWN_EMAIL = 해당 이메일 주소가 등록된 계정이 없습니다.
  .UNKNOWN_USER = 해당 사용자명을 가진 계정이 없습니다.
  .WRONG_PASSWORD = 비밀번호를 잘못 입력했습니다.

error-418 = 저는 찻주전자입니다.
