### Base localization / generic strings
### Messages should only go into this file if they are widely used,
### or are particularly important to localize.

## Terms

-service-name = Wikijump

## Special

# Shows whenever a message is still being loaded
message-loading = Loading...

goto-home = Go to home page

goto-service = Go to { -service-name }

base-title = { $title } | { -service-name }

navigated-to = Navigated to { $path }

## Generic

about = About
account = Account
alt-title = Alternative Title
applications = Applications
avatar = Avatar
breadcrumbs = Breadcrumbs
cancel = Cancel
change = Change
clear = clear
close = Close
dashboard = Dashboard
delete = Delete
docs = Docs
download = Download
edit = Edit
editor = Editor
footer = Page Footer
general = General
header = Page Header
help = Help
history = History
inbox = Inbox
invitations = Invitations
license = License
load = Load
main-content = Main Content
messages = Messages
move = Move
navigation = Navigation
notifications = Notifications
preview = Preview
privacy = Privacy
profile = Profile
publish = Publish
reveal-sidebar = Reveal Sidebar
save = Save
security = Security
send = Send
sent = Sent
settings = Settings
sidebar = Sidebar
tags = Tags
terms = Terms
title = Title
upload = Upload
view = View
vote = Vote

search = Search
  .placeholder = Search...

## Generic Authentication

login = Login
  .toast = You have been logged in.

logout = Logout
  .toast = You have been logged out.

register = Register
  .toast = You have been registered.

specifier = Email or Username
  .placeholder = Enter email or username...

username = Username
  .placeholder = Enter username...
  .info = You will be able to change this later.

email = Email
  .placeholder = Enter email address...
  .info = Your email address is private.

password = Password
  .placeholder = Enter password...

confirm-password = Confirm Password

forgot-password = Forgot Password
  .question = Forgot your password?

reset-password = Reset Password

remember-me = Remember me

create-account = Create Account

field-required = This field is required

characters-left = { $count ->
  [1] 1 character left
  *[other] { $count } characters left
}

hold-to-show-password = Hold to show password

## Errors

error-404 =
  .generic = The requested resource was not found.
  .page = The requested page was not found.
  .user = The requested user was not found.

error-form =
  .missing-fields = Please fill in all the required fields.
  .password-mismatch = Passwords do not match.

error-api =
  .GENERIC = Something went wrong with your request.
  .INTERNAL = An internal server error has occurred. Please try again later.
  .NO_CONNECTION = You are not connected to the internet.
  .BAD_SYNTAX = The request could not be understood by the server.
  .FORBIDDEN = You are not authorized to perform this action.
  .NOT_FOUND = The requested resource was not found.
  .CONFLICT = The requested resource is in conflict with another resource.

  .ACCOUNT_ALREADY_VERIFIED = This account has already been verified.
  .ACCOUNT_NO_EMAIL = This account does not have an email address.
  .ALREADY_LOGGED_IN = You are already logged in.
  .FAILED_TO_UPDATE_PROFILE = Failed to update profile.
  .INVALID_AVATAR = The uploaded file is not a valid image.
  .INVALID_EMAIL = The email address is invalid.
  .INVALID_LANGUAGE_CODE = The language code is invalid.
  .INVALID_PASSWORD = The password is invalid.
  .INVALID_SESSION = Your session has expired. Please log in again.
  .INVALID_SPECIFIER = The email or username is invalid.
  .INVALID_USERNAME = The username is invalid.
  .LOGIN_FAILED = Failed to log in. Please check your credentials.
  .NOT_LOGGED_IN = You are not logged in.
  .UNKNOWN_EMAIL = There is no account with that email address.
  .UNKNOWN_USER = There is no account with that username.
  .WRONG_PASSWORD = The password is incorrect.

error-418 = I'm a Teapot
