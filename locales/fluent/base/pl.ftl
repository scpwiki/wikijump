### Base localization / generic strings
### Messages should only go into this file if they are widely used,
### or are particularly important to localize.

## Terms

-service-name = Wikijump

## Special

# Shows whenever a message is still being loaded
message-loading = Ładowanie...

goto-home = Wróć na stronę główną

goto-service = Wejdź na { -service-name }

base-title = { $title } | { -service-name }

navigated-to = Przejdź do { $path }

## Generic

about = O stronie
account = Konto
applications = Aplikacje
avatar = Awatar
breadcrumbs = Nawigacja okruszkowa
change = Zmiany
clear = Wyczyść
close = Zamknij
dashboard = Dashboard
docs = Dokumentacja
download = Pobierz
edit = Edytuj
editor = Edytor
footer = Stopka strony
general = Główne
header = Nagłówek strony
help = Pomoc
inbox = Skrzynka odbiorcza
invitations = Zaproszenia
license = Licencja
load = Ładuj
main-content = Główna zawartość
messages = Wiadomość
navigation = Nawigacja
notifications = Powiadomienia
preview = Podgląd
privacy = Prywatność
profile = Profil
publish = Publikacja
reveal-sidebar = Rozwiń pasek boczny
save = Zapisz
security = Bezpieczeństwo
send = Wyślij
sent = Wysłano
settings = Ustawienia
sidebar = Pasek boczny
tags = Tagi
terms = Warunki
upload = Przekaż plik

search = Szukaj
  .placeholder = Wyszukiwanie...

## Generic Authentication

login = Zaloguj się
  .toast = Zostałeś zalogowany.

logout = Wyloguj się
  .toast = Zostałeś wylogowany.

register = Zarejestruj się
  .toast = Utworzyłeś konto.

specifier = Email lub Nazwa Użytkownika
  .placeholder = Wprowadź email lub nazwę użytkownika...

username = Nazwa użytkownika
  .placeholder = Wprowadź nazwę użytkownika...
  .info = Możesz to później zmienić.

email = Email
  .placeholder = Wprowadź adres email...
  .info = Twój adres email jest prywatny.

password = Hasło
  .placeholder = Wprowadź hasło...

confirm-password = Potwierdź hasło

forgot-password = Zapomniałem hasła
  .question = Zapomniałeś hasła?

reset-password = Resetuj hasło

remember-me = Zapamiętaj mnie

create-account = Utwórz konto

field-required = To pole jest wymagane

characters-left = { $count ->
  [1] Pozostał 1 znak
  *[other] pozostało { $count } znaków
}

hold-to-show-password = Przytrzymaj, aby pokazać hasło

## Errors

error-404 =
  .generic = Żądana treść nie została odnaleziona.
  .page = Wybrana strona nie została odnaleziona.
  .user = Wybrany użytkownik nie został odnaleziony.

error-form =
  .missing-fields = Proszę wypełnij wszystkie wymagane pola.
  .password-mismatch = Hasła się nie zgadzają.

error-api =
  .GENERIC = Coś poszło nie tak.
  .INTERNAL = Wystąpił problem wewnętrzny serwera. Proszę spróbować ponownie później.
  .NO_CONNECTION = Brak połączenia internetowego.
  .BAD_SYNTAX = Żądanie nie zostało rozpoznane przez serwer.
  .FORBIDDEN = Nie jesteś upoważniony do wykonania tej czynności.
  .NOT_FOUND = Żądana treść nie została odnaleziona.
  .CONFLICT = Żądana treść jest w konflikcie z inną treścią.

  .ACCOUNT_ALREADY_VERIFIED = Konto zostało już wcześniej zweryfikowane.
  .ACCOUNT_NO_EMAIL = Konto nie ma załączonego adresu email.
  .ALREADY_LOGGED_IN = Już jesteś zalogowany.
  .FAILED_TO_UPDATE_PROFILE = Nie udało się aktualizować profilu.
  .INVALID_AVATAR = Wczytywany plik nie jest prawidłowym zdjęciem.
  .INVALID_EMAIL = Nieprawidłowy adres email.
  .INVALID_LANGUAGE_CODE = Język kodu źródłowego jest nieprawidłowy.
  .INVALID_PASSWORD = Nieprawidłowe hasło.
  .INVALID_SESSION = Sesja wygasła. Zaloguj się ponownie.
  .INVALID_SPECIFIER = Nieprawidłowy adres email lub nazwa użytkownika.
  .INVALID_USERNAME = Nieprawdiłowa nazwa użytkownika.
  .LOGIN_FAILED = Nie udało się zalogować. Sprawdź swoje dane konta.
  .NOT_LOGGED_IN = Nie jesteś zalogowany.
  .UNKNOWN_EMAIL = Nie istnieje konto o takim adresie email.
  .UNKNOWN_USER = Nie istnieje konto o takiej nazwie użytkownika.
  .WRONG_PASSWORD = Nieprawidłowe hasło.

error-418 = Jestem czajnikiem
