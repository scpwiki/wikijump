
INSERT INTO site VALUES (1, 'My Wiki', 'powered by wikidot software', 'www', '', 'en', NULL, NULL, true, 'start', false, false);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

INSERT INTO site_settings VALUES (1, false, false, '', 314572800, false, 'system:join', 50, 20, true, NULL, false, false, 10485760, true);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

INSERT INTO site_super_settings VALUES (1, true);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

INSERT INTO admin VALUES(1, 1, 1, true);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

INSERT INTO member VALUES(1, 1, 1, NOW());;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

INSERT INTO category VALUES (1,
    1, '_default',
    false, 20,
    false, 'e:rm;c:rm;m:rm;d:rm;a:rm;r:rm;z:rm;o:rm',
    true, 1, NULL,
    false, 'nav:top', 'nav:side',
    NULL, false, true, NULL, NULL, NULL, true, false, false, NULL
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

INSERT INTO category VALUES (2,
    1, 'system',
    true, 20,
    false, 'e:;c:;m:;d:;a:;r:;z:;o:',
    true, 1, NULL,
    true, 'nav:top', 'nav:side',
    NULL, NULL, true, NULL, NULL, NULL, true, false, false, NULL
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

INSERT INTO category VALUES (3,
    1, 'search',
    true, 20,
    false, 'e:;c:;m:;d:;a:;r:;z:;o:',
    true, 1, NULL,
    true, 'nav:top', 'nav:side',
    NULL, NULL, true, NULL, NULL, NULL, true, false, false, NULL
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

INSERT INTO category VALUES (4,
    1, 'admin',
    false, 21,
    false, 'e:;c:;m:;d:;a:;r:;z:;o:',
    true, 1, NULL,
    false, 'nav:top', NULL,
    NULL, NULL, true, NULL, NULL, NULL, true, false, false, NULL
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

INSERT INTO category VALUES (5,
    1, 'auth',
    true, 20,
    false, 'e:;c:;m:;d:;a:;r:;z:;o:',
    true, 1, NULL,
    true, 'nav:top', 'nav:side',
    NULL, false, true, NULL, NULL, NULL, true, false, false, NULL
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

INSERT INTO category VALUES (6,
    1, 'account',
    false, 21,
    false, 'e:;c:;m:;d:;a:;r:;z:;o:',
    true, 1, NULL,
    false, 'nav:top', NULL,
    NULL, false, true, NULL, NULL, NULL, true, false, false, NULL
);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

