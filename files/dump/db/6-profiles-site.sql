
INSERT INTO site VALUES (3, 'User Profiles', '', 'profiles', '', 'en', NULL, NULL, true, 'start', false, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

INSERT INTO site_settings VALUES (3, false, false, '', 314572800, false, 'system:join', 50, 20, true, NULL, false, false, 10485760, true);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

INSERT INTO site_super_settings VALUES (3, true);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

INSERT INTO admin VALUES (3, 3, 1, true);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

INSERT INTO member VALUES (3, 3, 1, NOW());;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

INSERT INTO category VALUES (21,
    3, '_default',
    false, 20,
    false, 'e:rm;c:rm;m:rm;d:rm;a:rm;r:rm;z:rm;o:rm',
    true, 1, NULL,
    false, 'nav:top', 'nav:side',
    NULL, false, true, NULL, NULL, NULL, true, false, false, NULL
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

INSERT INTO category VALUES (22,
    3, 'system',
    true, 20,
    false, 'e:;c:;m:;d:;a:;r:;z:;o:',
    true, 1, NULL,
    true, 'nav:top', 'nav:side',
    NULL, NULL, true, NULL, NULL, NULL, true, false, false, NULL
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

INSERT INTO category VALUES (23,
    3, 'search',
    true, 20,
    false, 'e:;c:;m:;d:;a:;r:;z:;o:',
    true, 1, NULL,
    true, 'nav:top', 'nav:side',
    NULL, NULL, true, NULL, NULL, NULL, true, false, false, NULL
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

INSERT INTO category VALUES (24,
    3, 'admin',
    false, 21,
    false, 'e:;c:;m:;d:;a:;r:;z:;o:',
    true, 1, NULL,
    false, 'nav:top', NULL,
    NULL, NULL, true, NULL, NULL, NULL, true, false, false, NULL
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

INSERT INTO category VALUES (25,
    3, 'profile',
    false, 20,
    false, 'e:rm;c:rm;m:rm;d:rm;a:rm;r:rm;z:rm;o:rm',
    true, 1, NULL,
    false, 'nav:top', 'nav:side',
    NULL, false, true, NULL, NULL, NULL, true, false, false, NULL
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

