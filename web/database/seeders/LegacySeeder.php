<?php

namespace Database\Seeders;

use Exception;
use Illuminate\Database\Seeder;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Schema;

class LegacySeeder extends Seeder
{
    /**
     * Run the database seeds.
     * Throws Exception if random_bytes() can't collect enough entropy.
     *
     * @return void
     * @throws Exception
     */
    public function run()
    {
        DB::unprepared(
            <<<STATEMENT
INSERT INTO public.admin (admin_id, site_id, user_id, founder) VALUES (1, 1, 1, true);
INSERT INTO public.admin (admin_id, site_id, user_id, founder) VALUES (2, 2, 1, true);

INSERT INTO public.category (category_id, site_id, name, theme_default, theme_id, permissions_default, permissions, license_default, license_id, license_other, nav_default, top_bar_page_name, side_bar_page_name, template_id, per_page_discussion, per_page_discussion_default, rating, category_template_id, theme_external_url, autonumerate, page_title_template) VALUES (6, 2, 'nav', true, 20, true, 'v:arm;e:;c:;m:;d:;a:;r:;z:;o:', true, 1, NULL, true, 'nav:top', 'nav:side', NULL, NULL, true, NULL, NULL, NULL, false, NULL);
INSERT INTO public.category (category_id, site_id, name, theme_default, theme_id, permissions_default, permissions, license_default, license_id, license_other, nav_default, top_bar_page_name, side_bar_page_name, template_id, per_page_discussion, per_page_discussion_default, rating, category_template_id, theme_external_url, autonumerate, page_title_template) VALUES (14, 2, 'search', true, 20, true, 'v:arm;e:;c:;m:;d:;a:;r:;z:;o:', true, 1, NULL, true, 'nav:top', 'nav:side', NULL, NULL, true, NULL, NULL, NULL, false, NULL);
INSERT INTO public.category (category_id, site_id, name, theme_default, theme_id, permissions_default, permissions, license_default, license_id, license_other, nav_default, top_bar_page_name, side_bar_page_name, template_id, per_page_discussion, per_page_discussion_default, rating, category_template_id, theme_external_url, autonumerate, page_title_template) VALUES (15, 1, 'nav', true, 20, true, 'v:arm;e:;c:;m:;d:;a:;r:;z:;o:', true, 1, NULL, true, 'nav:top', 'nav:side', NULL, NULL, true, NULL, NULL, NULL, false, NULL);
INSERT INTO public.category (category_id, site_id, name, theme_default, theme_id, permissions_default, permissions, license_default, license_id, license_other, nav_default, top_bar_page_name, side_bar_page_name, template_id, per_page_discussion, per_page_discussion_default, rating, category_template_id, theme_external_url, autonumerate, page_title_template) VALUES (2, 2, '_default', true, 20, false, 'e:m;c:m;m:m;d:;a:m;r:m;z:;o:arm', false, 1, NULL, false, 'nav:top', 'nav:side', NULL, false, true, NULL, NULL, NULL, false, NULL);
INSERT INTO public.category (category_id, site_id, name, theme_default, theme_id, permissions_default, permissions, license_default, license_id, license_other, nav_default, top_bar_page_name, side_bar_page_name, template_id, per_page_discussion, per_page_discussion_default, rating, category_template_id, theme_external_url, autonumerate, page_title_template) VALUES (13, 2, 'admin', false, 21, false, 'v:arm;e:;c:;m:;d:;a:;r:;z:;o:', true, 1, NULL, true, 'nav:top', 'nav:side', NULL, NULL, true, NULL, NULL, NULL, false, NULL);
INSERT INTO public.category (category_id, site_id, name, theme_default, theme_id, permissions_default, permissions, license_default, license_id, license_other, nav_default, top_bar_page_name, side_bar_page_name, template_id, per_page_discussion, per_page_discussion_default, rating, category_template_id, theme_external_url, autonumerate, page_title_template) VALUES (17, 2, 'forum', true, 20, false, 'v:arm;e:;c:;m:;d:;a:;r:;z:;o:', true, 1, NULL, true, 'nav:top', 'nav:side', NULL, NULL, true, NULL, NULL, NULL, false, NULL);
INSERT INTO public.category (category_id, site_id, name, theme_default, theme_id, permissions_default, permissions, license_default, license_id, license_other, nav_default, top_bar_page_name, side_bar_page_name, template_id, per_page_discussion, per_page_discussion_default, rating, category_template_id, theme_external_url, autonumerate, page_title_template) VALUES (12, 2, 'system', true, 20, false, 'v:arm;e:;c:;m:;d:;a:;r:;z:;o:', true, 1, NULL, true, 'nav:top', 'nav:side', NULL, NULL, true, NULL, NULL, NULL, false, NULL);
INSERT INTO public.category (category_id, site_id, name, theme_default, theme_id, permissions_default, permissions, license_default, license_id, license_other, nav_default, top_bar_page_name, side_bar_page_name, template_id, per_page_discussion, per_page_discussion_default, rating, category_template_id, theme_external_url, autonumerate, page_title_template) VALUES (1, 1, '_default', true, 20, false, 'e:m;c:m;m:m;d:;a:m;r:m;z:;o:arm', false, 1, NULL, false, 'nav:top', 'nav:side', NULL, false, true, NULL, NULL, NULL, false, NULL);
INSERT INTO public.category (category_id, site_id, name, theme_default, theme_id, permissions_default, permissions, license_default, license_id, license_other, nav_default, top_bar_page_name, side_bar_page_name, template_id, per_page_discussion, per_page_discussion_default, rating, category_template_id, theme_external_url, autonumerate, page_title_template) VALUES (4, 1, 'account', false, 21, false, 'v:arm;e:;c:;m:;d:;a:;r:;z:;o:', true, 1, NULL, true, 'nav:top', 'nav:side', NULL, NULL, true, NULL, NULL, NULL, false, NULL);
INSERT INTO public.category (category_id, site_id, name, theme_default, theme_id, permissions_default, permissions, license_default, license_id, license_other, nav_default, top_bar_page_name, side_bar_page_name, template_id, per_page_discussion, per_page_discussion_default, rating, category_template_id, theme_external_url, autonumerate, page_title_template) VALUES (3, 1, 'admin', false, 21, false, 'v:arm;e:;c:;m:;d:;a:;r:;z:;o:', true, 1, NULL, true, 'nav:top', 'nav:side', NULL, NULL, true, NULL, NULL, NULL, false, NULL);
INSERT INTO public.category (category_id, site_id, name, theme_default, theme_id, permissions_default, permissions, license_default, license_id, license_other, nav_default, top_bar_page_name, side_bar_page_name, template_id, per_page_discussion, per_page_discussion_default, rating, category_template_id, theme_external_url, autonumerate, page_title_template) VALUES (16, 1, 'search', true, 20, false, 'v:arm;e:;c:;m:;d:;a:;r:;z:;o:', true, 1, NULL, true, 'nav:top', 'nav:side', NULL, NULL, true, NULL, NULL, NULL, false, NULL);
INSERT INTO public.category (category_id, site_id, name, theme_default, theme_id, permissions_default, permissions, license_default, license_id, license_other, nav_default, top_bar_page_name, side_bar_page_name, template_id, per_page_discussion, per_page_discussion_default, rating, category_template_id, theme_external_url, autonumerate, page_title_template) VALUES (5, 1, 'user', false, 21, false, 'v:arm;e:;c:;m:;d:;a:;r:;z:;o:', true, 1, NULL, true, 'nav:top', 'nav:side', NULL, NULL, true, NULL, NULL, NULL, false, NULL);
INSERT INTO public.category (category_id, site_id, name, theme_default, theme_id, permissions_default, permissions, license_default, license_id, license_other, nav_default, top_bar_page_name, side_bar_page_name, template_id, per_page_discussion, per_page_discussion_default, rating, category_template_id, theme_external_url, autonumerate, page_title_template) VALUES (18, 2, 'profile', true, 20, true, 'e:m;c:m;m:m;d:;a:m;r:m;z:;o:arm', true, 1, NULL, true, 'nav:top', 'nav:side', NULL, NULL, true, NULL, NULL, NULL, false, NULL);
INSERT INTO public.category (category_id, site_id, name, theme_default, theme_id, permissions_default, permissions, license_default, license_id, license_other, nav_default, top_bar_page_name, side_bar_page_name, template_id, per_page_discussion, per_page_discussion_default, rating, category_template_id, theme_external_url, autonumerate, page_title_template) VALUES (19, 1, 'system-all', true, 20, true, 'e:m;c:m;m:m;d:;a:m;r:m;z:;o:arm', true, 1, NULL, true, 'nav:top', 'nav:side', NULL, NULL, true, NULL, NULL, NULL, false, NULL);
INSERT INTO public.category (category_id, site_id, name, theme_default, theme_id, permissions_default, permissions, license_default, license_id, license_other, nav_default, top_bar_page_name, side_bar_page_name, template_id, per_page_discussion, per_page_discussion_default, rating, category_template_id, theme_external_url, autonumerate, page_title_template) VALUES (20, 1, 'system', true, 20, true, 'e:m;c:m;m:m;d:;a:m;r:m;z:;o:arm', true, 1, NULL, true, 'nav:top', 'nav:side', NULL, NULL, true, NULL, NULL, NULL, false, NULL);
INSERT INTO public.category (category_id, site_id, name, theme_default, theme_id, permissions_default, permissions, license_default, license_id, license_other, nav_default, top_bar_page_name, side_bar_page_name, template_id, per_page_discussion, per_page_discussion_default, rating, category_template_id, theme_external_url, autonumerate, page_title_template) VALUES (21, 1, 'auth', true, 20, false, 'e:m;c:m;m:m;d:;a:m;r:m;z:;o:arm', false, 1, NULL, false, 'nav:top', 'nav:side', NULL, false, true, NULL, NULL, NULL, false, NULL);

INSERT INTO public.forum_settings (site_id, permissions, per_page_discussion, max_nest_level) VALUES (2, 't:m;p:m;e:o;s:', false, 2);

INSERT INTO public.license (license_id, name, description, sort) VALUES (1, 'Creative Commons Attribution-ShareAlike 4.0 License (recommended)', '%%UNLESS%% <a rel="license" href="http://creativecommons.org/licenses/by-sa/4.0/">Creative Commons Attribution-ShareAlike 4.0 License</a>', 1);
INSERT INTO public.license (license_id, name, description, sort) VALUES (2, 'Creative Commons Attribution 4.0 License', '%%UNLESS%% <a rel="license" href="http://creativecommons.org/licenses/by/4.0/">Creative Commons Attribution 4.0 License</a>', 2);
INSERT INTO public.license (license_id, name, description, sort) VALUES (3, 'Creative Commons Attribution-NoDerivs 4.0 License', '%%UNLESS%% <a rel="license" href="http://creativecommons.org/licenses/by-nd/4.0/">Creative Commons Attribution-NoDerivs 4.0 License</a>', 3);
INSERT INTO public.license (license_id, name, description, sort) VALUES (4, 'Creative Commons Attribution-NonCommercial 4.0 License', '%%UNLESS%% <a rel="license" href="http://creativecommons.org/licenses/by-nc/4.0/">Creative Commons Attribution-NonCommercial 4.0 License</a>', 4);
INSERT INTO public.license (license_id, name, description, sort) VALUES (5, 'Creative Commons Attribution-NonCommercial-ShareAlike 4.0 License', '%%UNLESS%% <a rel="license" href="http://creativecommons.org/licenses/by-nc-sa/4.0/">Creative Commons Attribution-NonCommercial-ShareAlike 4.0 License</a>', 5);
INSERT INTO public.license (license_id, name, description, sort) VALUES (6, 'Creative Commons Attribution-NonCommercial-NoDerivs 4.0 License', '%%UNLESS%% <a rel="license" href="http://creativecommons.org/licenses/by-nc-nd/4.0/">Creative Commons Attribution-NonCommercial-NoDerivs 4.0 License</a>', 6);
INSERT INTO public.license (license_id, name, description, sort) VALUES (7, 'Creative Commons Attribution-ShareAlike 3.0 License', '%%UNLESS%% <a rel="license" href="http://creativecommons.org/licenses/by-sa/3.0/">Creative Commons Attribution-ShareAlike 3.0 License</a>', 7);
INSERT INTO public.license (license_id, name, description, sort) VALUES (8, 'Creative Commons Attribution 3.0 License', '%%UNLESS%% <a rel="license" href="http://creativecommons.org/licenses/by/3.0/">Creative Commons Attribution 3.0 License</a>', 8);
INSERT INTO public.license (license_id, name, description, sort) VALUES (9, 'Creative Commons Attribution-NoDerivs 3.0 License', '%%UNLESS%% <a rel="license" href="http://creativecommons.org/licenses/by-nd/3.0/">Creative Commons Attribution-NoDerivs 3.0 License</a>', 9);
INSERT INTO public.license (license_id, name, description, sort) VALUES (10, 'Creative Commons Attribution-NonCommercial 3.0 License', '%%UNLESS%% <a rel="license" href="http://creativecommons.org/licenses/by-nc/3.0/">Creative Commons Attribution-NonCommercial 3.0 License</a>', 10);
INSERT INTO public.license (license_id, name, description, sort) VALUES (11, 'Creative Commons Attribution-NonCommercial-ShareAlike 3.0 License', '%%UNLESS%% <a rel="license" href="http://creativecommons.org/licenses/by-nc-sa/3.0/">Creative Commons Attribution-NonCommercial-ShareAlike 3.0 License</a>', 11);
INSERT INTO public.license (license_id, name, description, sort) VALUES (12, 'Creative Commons Attribution-NonCommercial-NoDerivs 3.0 License', '%%UNLESS%% <a rel="license" href="http://creativecommons.org/licenses/by-nc-nd/3.0/">Creative Commons Attribution-NonCommercial-NoDerivs 3.0 License</a>', 12);
INSERT INTO public.license (license_id, name, description, sort) VALUES (13, 'CC0 (Public Domain)', '%%UNLESS%%
<a rel="license" href="https://creativecommons.org/publicdomain/zero/1.0/">CC0 (Public Domain)</a>.', 100);
INSERT INTO public.license (license_id, name, description, sort) VALUES (14, 'GNU Free Documentation License 1.3', '%%UNLESS%%
<a rel="license" href="http://www.gnu.org/copyleft/fdl.html">GNU
Free Documentation License</a>.', 101);
INSERT INTO public.license (license_id, name, description, sort) VALUES (15, 'Standard copyright (not recommended)', NULL, 1000);

INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (1, 1, 3, NULL, 1, 1, 1, 0, NULL, 'admin:manage', '2008-01-24 12:16:34', '2008-01-24 12:16:34', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (2, 1, 4, NULL, 2, 2, 2, 0, NULL, 'account:you', '2008-01-24 12:22:02', '2008-01-24 12:22:02', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (3, 1, 1, NULL, 3, 3, 3, 0, 'Get a new wiki', 'new-site', '2008-01-24 12:27:10', '2008-01-24 12:27:10', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (4, 1, 5, NULL, 4, 4, 4, 0, NULL, 'user:info', '2008-01-24 12:32:21', '2008-01-24 12:32:21', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (5, 2, 6, NULL, 5, 5, 5, 0, 'Side', 'nav:side', '2008-01-25 00:35:20', '2008-01-25 00:35:20', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (6, 2, 2, NULL, 6, 6, 6, 0, 'What Is A Wiki Site', 'what-is-a-wiki-site', '2008-01-25 00:45:30', '2008-01-25 00:45:30', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (14, 2, 12, NULL, 15, 15, 14, 0, 'Join This Wiki', 'system:join', '2008-01-29 00:56:59', '2008-01-29 00:56:59', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (15, 2, 13, NULL, 16, 16, 15, 0, NULL, 'admin:manage', '2008-01-29 00:57:39', '2008-01-29 00:57:39', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (16, 2, 12, NULL, 17, 17, 16, 0, 'Page Tags List', 'system:page-tags-list', '2008-01-29 00:58:44', '2008-01-29 00:58:44', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (17, 2, 12, NULL, 18, 18, 17, 0, 'Recent Changes', 'system:recent-changes', '2008-01-29 00:59:14', '2008-01-29 00:59:14', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (18, 2, 12, NULL, 19, 19, 18, 0, 'Members', 'system:members', '2008-01-29 00:59:40', '2008-01-29 00:59:40', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (19, 2, 14, NULL, 20, 20, 19, 0, 'Wiki Search', 'search:site', '2008-01-29 01:01:49', '2008-01-29 01:01:49', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (20, 2, 12, NULL, 21, 21, 20, 0, NULL, 'system:page-tags', '2008-01-29 01:03:43', '2008-01-29 01:03:43', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (21, 2, 12, NULL, 22, 22, 21, 0, 'List All Pages', 'system:list-all-pages', '2008-01-29 01:04:52', '2008-01-29 01:04:52', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (24, 1, 16, NULL, 25, 25, 24, 0, 'Search All Wikis', 'search:all', '2008-01-29 01:09:17', '2008-01-29 01:09:17', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (25, 1, 16, NULL, 27, 26, 26, 1, 'Search This Wiki', 'search:site', '2008-01-29 01:34:40', '2008-01-29 01:34:57', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (26, 1, 16, NULL, 30, 29, 27, 1, 'Search Users', 'search:users', '2008-01-29 01:36:56', '2008-01-29 01:37:12', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (27, 2, 17, NULL, 31, 30, 28, 0, 'Forum Categories', 'forum:start', '2008-01-29 01:40:23', '2008-01-29 01:40:23', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (28, 2, 17, NULL, 32, 31, 29, 0, 'Forum Category', 'forum:category', '2008-01-29 01:40:59', '2008-01-29 01:40:59', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (29, 2, 17, NULL, 33, 32, 30, 0, 'Forum Thread', 'forum:thread', '2008-01-29 01:41:32', '2008-01-29 01:41:32', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (30, 2, 17, NULL, 34, 33, 31, 0, 'New Forum Thread', 'forum:new-thread', '2008-01-29 01:42:10', '2008-01-29 01:42:10', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (31, 2, 17, NULL, 35, 34, 32, 0, 'Recent Forum Posts', 'forum:recent-posts', '2008-01-29 01:42:42', '2008-01-29 01:42:42', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (32, 2, 6, NULL, 36, 35, 33, 0, 'Top', 'nav:top', '2008-01-29 23:29:51', '2008-01-29 23:29:51', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (33, 2, 18, NULL, 37, 36, 34, 0, 'Template', 'profile:template', '2008-01-29 23:30:18', '2008-01-29 23:30:18', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (36, 2, 2, NULL, 40, 39, 37, 0, 'Congratulations, welcome to your new wiki!', 'start', '2008-01-30 08:43:22', '2008-01-30 08:43:22', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (37, 1, 19, NULL, 42, 41, 38, 0, 'List of all wikis', 'system-all:all-sites', '2008-01-30 08:54:56', '2008-01-30 08:54:56', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (38, 1, 19, NULL, 44, 43, 40, 1, 'List wikis by tags', 'system-all:sites-by-tags', '2008-01-30 08:55:33', '2008-01-30 09:00:00', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (22, 1, 15, NULL, 45, 44, 22, 2, 'Side', 'nav:side', '2008-01-29 01:05:47', '2008-01-30 09:01:50', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (39, 1, 19, NULL, 46, 45, 41, 0, 'Search', 'system-all:search', '2008-01-30 09:07:05', '2008-01-30 09:07:05', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (40, 1, 19, NULL, 48, 47, 43, 1, 'Activity across all wikis', 'system-all:activity', '2008-01-30 09:16:38', '2008-01-30 09:17:40', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (23, 1, 1, NULL, 50, 49, 44, 3, 'Welcome to your new Wikidot installation!', 'start', '2008-01-29 01:07:41', '2008-01-30 16:08:02', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (41, 1, 1, NULL, 51, 50, 45, 0, 'What Is A Wiki', 'what-is-a-wiki', '2008-01-30 16:11:56', '2008-01-30 16:11:56', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (13, 2, 2, NULL, 52, 51, 13, 1, 'How To Edit Pages - Quickstart', 'how-to-edit-pages', '2008-01-29 00:09:59', '2008-01-30 16:12:40', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (42, 1, 1, NULL, 53, 52, 46, 0, 'How To Edit Pages', 'how-to-edit-pages', '2008-01-30 16:12:48', '2008-01-30 16:12:48', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (43, 1, 20, NULL, 54, 53, 47, 0, 'Wiki Members', 'system:members', '2008-01-30 16:13:32', '2008-01-30 16:13:32', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (44, 1, 20, NULL, 55, 54, 48, 0, 'How to join this wiki?', 'system:join', '2008-01-30 16:14:13', '2008-01-30 16:14:13', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (45, 1, 20, NULL, 56, 55, 49, 0, 'Recent changes', 'system:recent-changes', '2008-01-30 16:14:41', '2008-01-30 16:14:41', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (46, 1, 20, NULL, 57, 56, 50, 0, 'List all pages', 'system:list-all-pages', '2008-01-30 16:15:22', '2008-01-30 16:15:22', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (47, 1, 20, NULL, 58, 57, 51, 0, 'Page Tags List', 'system:page-tags-list', '2008-01-30 16:15:56', '2008-01-30 16:15:56', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (48, 1, 20, NULL, 59, 58, 52, 0, 'Page Tags', 'system:page-tags', '2008-01-30 16:16:22', '2008-01-30 16:16:22', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (49, 1, 21, NULL, 60, 59, 53, 0, 'Log in', 'auth:login', '2008-08-19 16:25:58', '2008-08-19 16:25:58', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (50, 1, 21, NULL, 61, 60, 54, 0, 'Create account - step 1', 'auth:newaccount', '2008-08-19 16:25:58', '2008-08-19 16:25:58', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (51, 1, 21, NULL, 62, 61, 55, 0, 'Create account - step 2', 'auth:newaccount2', '2008-08-19 16:25:58', '2008-08-19 16:25:58', 1, NULL, NULL, 1, false, 0);
INSERT INTO public.page (page_id, site_id, category_id, parent_page_id, revision_id, source_id, metadata_id, revision_number, title, unix_name, date_created, date_last_edited, last_edit_user_id, last_edit_user_string, thread_id, owner_user_id, blocked, rate) VALUES (52, 1, 21, NULL, 63, 62, 56, 0, 'Create account - step 3', 'auth:newaccount3', '2008-08-19 16:25:58', '2008-08-19 16:25:58', 1, NULL, NULL, 1, false, 0);

INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (6, '

<p>According to <a href="http://en.wikipedia.org/wiki/Wiki">Wikipedia</a>, the world largest wiki site:</p>
<blockquote>
<p>A <em>Wiki</em> ([ˈwiː.kiː] &lt;wee-kee&gt; or [ˈwɪ.kiː] &lt;wick-ey&gt;) is a type of website that allows users to add, remove, or otherwise edit and change most content very quickly and easily.</p>
</blockquote>
<p>And that is it! As a part of a farm of wikis this site is a great tool that you can use to publish content, upload files, communicate and collaborate.</p>
', '2008-01-25 00:45:30');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (15, '

þmodule "ManageSite/ManageSiteModule"þ', '2008-01-29 00:57:39');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (14, '

<div class="wiki-note">
<p>Please change this page according to your policy (configure first using <a href="/admin:manage">Site Manager</a>) and remove this note.</p>
</div>
<h1 id="toc0"><span>Who can join?</span></h1>
<p>You can write here who can become a member of this site.</p>
<h1 id="toc1"><span>Join!</span></h1>
<p>So you want to become a member of this site? Tell us why and apply now!</p>
þmodule "Membership/MembershipApplyModule"þ<br />
<p>Or, if you already know a "secret password", go for it!</p>
þmodule "Membership/MembershipByPasswordModule"þ', '2008-01-29 00:57:39');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (16, '

þmodule "Wiki/PagesTagCloud/PagesTagCloudModule" limit%3D%22200%22+target%3D%22system%3Apage-tags%22 þ', '2008-01-29 00:58:44');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (17, '

þmodule "Changes/SiteChangesModule"þ', '2008-01-29 00:59:15');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (18, '

<h1 id="toc0"><span>Members:</span></h1>
þmodule "Membership/MembersListModule"þ
<h1 id="toc1"><span>Moderators</span></h1>
þmodule "Membership/MembersListModule" group%3D%22moderators%22 þ
<h1 id="toc2"><span>Admins</span></h1>
þmodule "Membership/MembersListModule" group%3D%22admins%22 þ', '2008-01-29 00:59:40');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (19, '

þmodule "Search/SearchModule"þ', '2008-01-29 01:01:49');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (20, '

<div style="float:right; width: 50%;">þmodule "Wiki/PagesTagCloud/PagesTagCloudModule" limit%3D%22200%22+target%3D%22system%3Apage-tags%22 þ</div>
þmodule "Wiki/PagesTagCloud/PagesListByTagModule"þ', '2008-01-29 01:03:43');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (21, '

þmodule "List/WikiPagesModule" preview%3D%22true%22 þ', '2008-01-29 01:04:52');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (27, '

þmodule "Forum/ForumStartModule"þ', '2008-01-29 01:40:24');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (28, '

þmodule "Forum/ForumViewCategoryModule"þ', '2008-01-29 01:40:59');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (29, '

þmodule "Forum/ForumViewThreadModule"þ', '2008-01-29 01:41:32');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (30, '

þmodule "Forum/ForumNewThreadModule"þ', '2008-01-29 01:42:10');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (31, '

þmodule "Forum/ForumRecentPostsModule"þ', '2008-01-29 01:42:42');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (32, '

<ul>
<li><a href="javascript:;">example menu</a>
<ul>
<li><a class="newpage" href="/submenu">submenu</a></li>
</ul>
</li>
<li><a class="newpage" href="/contact">contact</a></li>
</ul>
', '2008-01-29 23:29:51');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (33, '

<p>Profile has not been created (yet).</p>
', '2008-01-29 23:30:18');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (5, '

<ul>
<li><a href="/start">Welcome page</a></li>
</ul>
<ul>
<li><a href="/what-is-a-wiki-site">What is a Wiki Site?</a></li>
<li><a href="/how-to-edit-pages">How to edit pages?</a></li>
</ul>
<ul>
<li><a href="/system:join">How to join this site?</a></li>
<li><a href="/system:members">Site members</a></li>
</ul>
<ul>
<li><a href="/system:recent-changes">Recent changes</a></li>
<li><a href="/system:list-all-pages">List all pages</a></li>
<li><a href="/system:page-tags-list">Page Tags</a></li>
</ul>
<ul>
<li><a href="/admin:manage">Site Manager</a></li>
</ul>
<h2 id="toc0"><span>Page tags</span></h2>
þmodule "Wiki/PagesTagCloud/PagesTagCloudModule" minFontSize%3D%2280%25%22+maxFontSize%3D%22200%25%22++maxColor%3D%228%2C8%2C64%22+minColor%3D%22100%2C100%2C128%22+target%3D%22system%3Apage-tags%22+limit%3D%2230%22 þ
<h2 id="toc1"><span>Add a new page</span></h2>
þmodule "Misc/NewPageHelperModule" size%3D%2215%22+button%3D%22new+page%22 þ
<p style="text-align: center;"><span style="font-size:80%;"><a href="/nav:side">edit this panel</a></span></p>
', '2008-01-30 08:39:25');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (36, '

<h2 id="toc0"><span>If this is your first site</span></h2>
<p>Then there are some things you need to know:</p>
<ul>
<li>You can configure all security and other settings online, using the <a href="/admin:manage">Site Manager</a>. When you invite other people to help build this site they don''t have access to the Site Manager unless you make them administrators like yourself. Check out the <em>Permissions</em> section.</li>
<li>Your Wikidot site has two menus, <a href="/nav:side">one at the side</a> called ''<tt>nav:side</tt>'', and <a href="/nav:top">one at the top</a> called ''<tt>nav:top</tt>''. These are Wikidot pages, and you can edit them like any page.</li>
<li>To edit a page, go to the page and click the <strong>Edit</strong> button at the bottom. You can change everything in the main area of your page. The Wikidot system is <a href="http://www.wikidot.org/doc" onclick="window.open(this.href, ''_blank''); return false;">easy to learn and powerful</a>.</li>
<li>You can attach images and other files to any page, then display them and link to them in the page.</li>
<li>Every Wikidot page has a history of edits, and you can undo anything. So feel secure, and experiment.</li>
<li>To start a forum on your site, see the <a href="/admin:manage">Site Manager</a> » <em>Forum</em>.</li>
<li>The license for this Wikidot site has been set to <a href="http://creativecommons.org/licenses/by-sa/3.0/" onclick="window.open(this.href, ''_blank''); return false;">Creative Commons Attribution-Share Alike 3.0 License</a>. If you want to change this, use the Site Manager.</li>
<li>If you want to learn more, make sure you visit the <a href="http://www.wikidot.org/doc" onclick="window.open(this.href, ''_blank''); return false;">Documentation section at www.wikidot.org</a></li>
</ul>
<p>More information about the Wikidot project can be found at <a href="http://www.wikidot.org" onclick="window.open(this.href, ''_blank''); return false;">www.wikidot.org</a>.</p>
', '2008-01-30 08:43:22');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (13, '

<p>If you are allowed to edit pages in this Site, simply click on <em>edit</em> button at the bottom of the page. This will open an editor with a toolbar pallette with options.</p>
<p>To create a link to a new page, use syntax: <tt>[[[new page name]]]</tt> or <tt>[[[new page name | text to display]]]</tt>. Follow the link (which should have a different color if page does not exist) and create a new page and edit it!</p>
<p>Although creating and editing pages is easy, there are a lot more options that allows creating powerful sites. Please visit <a href="http://www.wikidot.org/doc" onclick="window.open(this.href, ''_blank''); return false;">Documentation pages</a> (at wikidot.org) to learn more.</p>
', '2008-01-30 16:12:40');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (45, '

þmodule "Changes/SiteChangesModule"þ', '2008-08-19 16:25:59');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (1, '

þmodule "ManageSite/ManageSiteModule"þ', '2008-08-19 16:25:58');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (2, '

þmodule "Account/AccountModule"þ', '2008-08-19 16:25:58');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (3, '

<p>Use this simple form to create a new wiki.</p>
<p>To admins: you can customize this page by simply clicking "edit" at the bottom of the page.</p>
þmodule "NewSite/NewSiteModule"þ', '2008-08-19 16:25:58');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (4, '

þmodule "UserInfo/UserInfoModule"þ', '2008-08-19 16:25:58');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (24, '

þmodule "Search/SearchAllModule"þ', '2008-08-19 16:25:58');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (25, '

þmodule "Search/SearchModule"þ', '2008-08-19 16:25:58');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (26, '

<p>To look for someone, please enter:</p>
<ul>
<li>email address of a person you are looking for (this will look for exact match)</li>
<li>any part of the screen name or realname (lists all Users matching the query)</li>
</ul>
þmodule "Search/UserSearchModule"þ', '2008-08-19 16:25:58');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (37, '

<p>Below is the list of public visible Wikis hosted at this service:</p>
þmodule "Wiki/ListAllWikis/ListAllWikisModule"þ', '2008-08-19 16:25:58');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (38, '

þmodule "Wiki/SitesTagCloud/SitesTagCloudModule" limit%3D%22100%22+target%3D%22system-all%3Asites-by-tags%22 þþmodule "Wiki/SitesTagCloud/SitesListByTagModule"þ', '2008-08-19 16:25:58');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (22, '

<ul>
<li><a href="/start">Welcome page</a></li>
</ul>
<ul>
<li><a href="/what-is-a-wiki">What is a Wiki?</a></li>
<li><a href="/how-to-edit-pages">How to edit pages?</a></li>
<li><a href="/new-site">Get a new wiki!</a></li>
</ul>
<h1 id="toc0"><span>All wikis</span></h1>
<ul>
<li><a href="/system-all:activity">Recent activity</a></li>
<li><a href="/system-all:all-sites">All wikis</a></li>
<li><a href="/system-all:sites-by-tags">Wikis by tags</a></li>
<li><a href="/system-all:search">Search</a></li>
</ul>
<h1 id="toc1"><span>This wiki</span></h1>
<ul>
<li><a href="/system:join">How to join this site?</a></li>
<li><a href="/system:members">Site members</a></li>
</ul>
<ul>
<li><a href="/system:recent-changes">Recent changes</a></li>
<li><a href="/system:list-all-pages">List all pages</a></li>
<li><a href="/system:page-tags-list">Page Tags</a></li>
</ul>
<ul>
<li><a href="/admin:manage">Site Manager</a></li>
</ul>
<h2 id="toc2"><span>Page tags</span></h2>
þmodule "Wiki/PagesTagCloud/PagesTagCloudModule" minFontSize%3D%2280%25%22+maxFontSize%3D%22200%25%22++maxColor%3D%228%2C8%2C64%22+minColor%3D%22100%2C100%2C128%22+target%3D%22system%3Apage-tags%22+limit%3D%2230%22 þ
<h2 id="toc3"><span>Add a new page</span></h2>
þmodule "Misc/NewPageHelperModule" size%3D%2215%22+button%3D%22new+page%22 þ
<p style="text-align: center;"><span style="font-size:80%;"><a href="/nav:side">edit this panel</a></span></p>
', '2008-08-19 16:25:58');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (39, '

<div style="text-align: center;">
<h1 id="toc0"><span>Search all Wikis</span></h1>
<p>Perform a search through all public and visible wikis.</p>
þmodule "Search/SearchAllModule"þ
<hr />
<h1 id="toc1"><span>Search users</span></h1>
<p>To look for someone, please enter:</p>
<ul>
<li>email address of a person you are looking for (this will look for exact match)</li>
<li>any part of the screen name or realname (lists all Users matching the query)</li>
</ul>
þmodule "Search/UserSearchModule"þ</div>
', '2008-08-19 16:25:59');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (40, '

<table>
<tr>
<td style="width: 45%; padding-right: 2%; border-right: 1px solid #999; vertical-align:top;">
<h2 id="toc0"><span>Recent edits (all wikis)</span></h2>
þmodule "Wiki/SitesActivity/RecentWPageRevisionsModule"þ</td>
<td style="width: 45%; padding-left: 2%; vertical-align:top;">
<h2 id="toc1"><span>Top Sites</span></h2>
þmodule "Wiki/SitesActivity/MostActiveSitesModule"þ
<h2 id="toc2"><span>Top Forums</span></h2>
þmodule "Wiki/sitesactivity/MostActiveForumsModule"þ
<h2 id="toc3"><span>New users</span></h2>
þmodule "Wiki/sitesactivity/NewWUsersModule"þ
<h2 id="toc4"><span>Some statistics</span></h2>
þmodule "Wiki/sitesactivity/SomeGlobalStatsModule"þ</td>
</tr>
</table>
', '2008-08-19 16:25:59');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (23, '

<p>Congratulations, you have successfully installed Wikidot software on your computer!</p>
<h1 id="toc0"><span>What to do next</span></h1>
<h2 id="toc1"><span>Customize this wiki</span></h2>
<p>Wikidot consists of several wiki sites, not just one. Right now you are on the main wiki. Customize it!</p>
<ul>
<li>You can configure all security and other settings online, using the <a href="/admin:manage">Site Manager</a>. When you invite other people to help build this site they don''t have access to the Site Manager unless you make them administrators like yourself. Check out the <em>Permissions</em> section.</li>
<li>Your Wikidot site has two menus, <a href="/nav:side">one at the side</a> called ''<tt>nav:side</tt>'', and <a class="newpage" href="/nav:top">one at the top</a> called ''<tt>nav:top</tt>''. These are Wikidot pages, and you can edit them like any page.</li>
<li>To edit a page, go to the page and click the <strong>Edit</strong> button at the bottom. You can change everything in the main area of your page. The Wikidot system is <a href="http://www.wikidot.org/doc" onclick="window.open(this.href, ''_blank''); return false;">easy to learn and powerful</a>.</li>
<li>You can attach images and other files to any page, then display them and link to them in the page.</li>
<li>Every Wikidot page has a history of edits, and you can undo anything. So feel secure, and experiment.</li>
<li>To start a forum on your site, see the <a href="/admin:manage">Site Manager</a> » <em>Forum</em>.</li>
<li>The license for this Wikidot site has been set to <a href="http://creativecommons.org/licenses/by-sa/3.0/" onclick="window.open(this.href, ''_blank''); return false;">Creative Commons Attribution-Share Alike 3.0 License</a>. If you want to change this, use the Site Manager.</li>
<li>If you want to learn more, make sure you visit the <a href="http://www.wikidot.org/doc" onclick="window.open(this.href, ''_blank''); return false;">Documentation section at www.wikidot.org</a></li>
</ul>
<h2 id="toc2"><span>Customize default template</span></h2>
<p>Default initial template for other wikis is located at <a href="http://template-en.wikidot1.dev/template-en">template-en</a>. If someone creates a new wiki, this one is cloned to the new address. A good thing to do is to go to <a href="http://template-en.wikidot1.dev/template-en">template-en</a> and customize it.</p>
<h2 id="toc3"><span>Create more templates</span></h2>
<p>Simply create wikis with unix names starting with "template-" (e.g. "template-pl", "template-blog") and your users will be able to choose which wiki they want to start with.</p>
<h2 id="toc4"><span>Visit Wikidot.org</span></h2>
<p>Go to <strong><a href="http://www.wikidot.org">www.wikidot.org</a></strong> — home of the Wikidot software — for extra documentation, howtos, tips and support.</p>
<hr />
<p>More information about the Wikidot project can be found at <a href="http://www.wikidot.org" onclick="window.open(this.href, ''_blank''); return false;">www.wikidot.org</a>.</p>
<h1 id="toc5"><span>Search all wikis</span></h1>
þmodule "Search/SearchAllModule"þ
<h1 id="toc6"><span>Search users</span></h1>
þmodule "Search/UserSearchModule"þ', '2008-08-19 16:25:59');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (41, '

<p>According to <a href="http://en.wikipedia.org/wiki/Wiki">Wikipedia</a>, the world largest wiki site:</p>
<blockquote>
<p>A <em>Wiki</em> ([ˈwiː.kiː] &lt;wee-kee&gt; or [ˈwɪ.kiː] &lt;wick-ey&gt;) is a type of website that allows users to add, remove, or otherwise edit and change most content very quickly and easily.</p>
</blockquote>
<p>And that is it! As a part of a farm of wikis this site is a great tool that you can use to publish content, upload files, communicate and collaborate.</p>
', '2008-08-19 16:25:59');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (42, '

<p>If you are allowed to edit pages in this Site, simply click on <em>edit</em> button at the bottom of the page. This will open an editor with a toolbar pallette with options.</p>
<p>To create a link to a new page, use syntax: <tt>[[[new page name]]]</tt> or <tt>[[[new page name | text to display]]]</tt>. Follow the link (which should have a different color if page does not exist) and create a new page and edit it!</p>
<p>Although creating and editing pages is easy, there are a lot more options that allows creating powerful sites. Please visit <a href="http://www.wikidot.org/doc" onclick="window.open(this.href, ''_blank''); return false;">Documentation pages</a> (at wikidot.org) to learn more.</p>
', '2008-08-19 16:25:59');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (43, '

<h1 id="toc0"><span>Members:</span></h1>
þmodule "Membership/MembersListModule"þ
<h1 id="toc1"><span>Moderators</span></h1>
þmodule "Membership/MembersListModule" group%3D%22moderators%22 þ
<h1 id="toc2"><span>Admins</span></h1>
þmodule "Membership/MembersListModule" group%3D%22admins%22 þ', '2008-08-19 16:25:59');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (44, '

<div class="wiki-note">
<p>Please change this page according to your policy (configure first using <a href="/admin:manage">Site Manager</a>) and remove this note.</p>
</div>
<h1 id="toc0"><span>Who can join?</span></h1>
<p>You can write here who can become a member of this site.</p>
<h1 id="toc1"><span>Join!</span></h1>
<p>So you want to become a member of this site? Tell us why and apply now!</p>
þmodule "Membership/MembershipApplyModule"þ
<p>Or, if you already know a "secret password", go for it!</p>
þmodule "Membership/MembershipByPasswordModule"þ', '2008-08-19 16:25:59');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (46, '

þmodule "List/WikiPagesModule" preview%3D%22true%22 þ', '2008-08-19 16:25:59');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (49, '

þmodule "Login/LoginModule"þ', '2008-08-19 16:25:59');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (47, '

þmodule "Wiki/PagesTagCloud/PagesTagCloudModule" limit%3D%22200%22+target%3D%22system%3Apage-tags%22 þ', '2008-08-19 16:25:59');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (48, '

<div style="float:right; width: 50%;">þmodule "Wiki/PagesTagCloud/PagesTagCloudModule" limit%3D%22200%22+target%3D%22system%3Apage-tags%22 þ</div>
þmodule "Wiki/PagesTagCloud/PagesListByTagModule"þ', '2008-08-19 16:25:59');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (50, '

þmodule "CreateAccount/CreateAccountStep1Module"þ', '2008-08-19 16:25:59');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (51, '

þmodule "CreateAccount/CreateAccountStep2Module"þ', '2008-08-19 16:25:59');
INSERT INTO public.page_compiled (page_id, text, date_compiled) VALUES (52, '

þmodule "CreateAccount/CreateAccountStep3Module"þ', '2008-08-19 16:25:59');

INSERT INTO public.page_link (link_id, from_page_id, to_page_id, to_page_name, site_id) VALUES (1, 5, 5, NULL, 2);
INSERT INTO public.page_link (link_id, from_page_id, to_page_id, to_page_name, site_id) VALUES (2, 5, 6, NULL, 2);
INSERT INTO public.page_link (link_id, from_page_id, to_page_id, to_page_name, site_id) VALUES (3, 5, 13, NULL, 2);
INSERT INTO public.page_link (link_id, from_page_id, to_page_id, to_page_name, site_id) VALUES (4, 5, 14, NULL, 2);
INSERT INTO public.page_link (link_id, from_page_id, to_page_id, to_page_name, site_id) VALUES (5, 14, 15, NULL, 2);
INSERT INTO public.page_link (link_id, from_page_id, to_page_id, to_page_name, site_id) VALUES (6, 5, 15, NULL, 2);
INSERT INTO public.page_link (link_id, from_page_id, to_page_id, to_page_name, site_id) VALUES (7, 5, 16, NULL, 2);
INSERT INTO public.page_link (link_id, from_page_id, to_page_id, to_page_name, site_id) VALUES (8, 5, 17, NULL, 2);
INSERT INTO public.page_link (link_id, from_page_id, to_page_id, to_page_name, site_id) VALUES (9, 5, 18, NULL, 2);
INSERT INTO public.page_link (link_id, from_page_id, to_page_id, to_page_name, site_id) VALUES (10, 5, 21, NULL, 2);
INSERT INTO public.page_link (link_id, from_page_id, to_page_id, to_page_name, site_id) VALUES (11, 22, 1, NULL, 1);
INSERT INTO public.page_link (link_id, from_page_id, to_page_id, to_page_name, site_id) VALUES (12, 22, 22, NULL, 1);
INSERT INTO public.page_link (link_id, from_page_id, to_page_id, to_page_name, site_id) VALUES (13, 22, 23, NULL, 1);
INSERT INTO public.page_link (link_id, from_page_id, to_page_id, to_page_name, site_id) VALUES (14, 32, NULL, 'submenu', 2);
INSERT INTO public.page_link (link_id, from_page_id, to_page_id, to_page_name, site_id) VALUES (15, 32, NULL, 'contact', 2);
INSERT INTO public.page_link (link_id, from_page_id, to_page_id, to_page_name, site_id) VALUES (16, 36, 15, NULL, 2);
INSERT INTO public.page_link (link_id, from_page_id, to_page_id, to_page_name, site_id) VALUES (17, 36, 5, NULL, 2);
INSERT INTO public.page_link (link_id, from_page_id, to_page_id, to_page_name, site_id) VALUES (18, 36, 32, NULL, 2);
INSERT INTO public.page_link (link_id, from_page_id, to_page_id, to_page_name, site_id) VALUES (19, 22, 37, NULL, 1);
INSERT INTO public.page_link (link_id, from_page_id, to_page_id, to_page_name, site_id) VALUES (20, 22, 38, NULL, 1);
INSERT INTO public.page_link (link_id, from_page_id, to_page_id, to_page_name, site_id) VALUES (21, 22, 3, NULL, 1);
INSERT INTO public.page_link (link_id, from_page_id, to_page_id, to_page_name, site_id) VALUES (22, 22, 39, NULL, 1);
INSERT INTO public.page_link (link_id, from_page_id, to_page_id, to_page_name, site_id) VALUES (23, 22, 40, NULL, 1);
INSERT INTO public.page_link (link_id, from_page_id, to_page_id, to_page_name, site_id) VALUES (24, 23, 1, NULL, 1);
INSERT INTO public.page_link (link_id, from_page_id, to_page_id, to_page_name, site_id) VALUES (25, 23, 22, NULL, 1);
INSERT INTO public.page_link (link_id, from_page_id, to_page_id, to_page_name, site_id) VALUES (26, 23, NULL, 'nav:top', 1);
INSERT INTO public.page_link (link_id, from_page_id, to_page_id, to_page_name, site_id) VALUES (27, 22, 41, NULL, 1);
INSERT INTO public.page_link (link_id, from_page_id, to_page_id, to_page_name, site_id) VALUES (28, 22, 42, NULL, 1);
INSERT INTO public.page_link (link_id, from_page_id, to_page_id, to_page_name, site_id) VALUES (29, 22, 43, NULL, 1);
INSERT INTO public.page_link (link_id, from_page_id, to_page_id, to_page_name, site_id) VALUES (30, 22, 44, NULL, 1);
INSERT INTO public.page_link (link_id, from_page_id, to_page_id, to_page_name, site_id) VALUES (31, 44, 1, NULL, 1);
INSERT INTO public.page_link (link_id, from_page_id, to_page_id, to_page_name, site_id) VALUES (32, 22, 45, NULL, 1);
INSERT INTO public.page_link (link_id, from_page_id, to_page_id, to_page_name, site_id) VALUES (33, 22, 46, NULL, 1);
INSERT INTO public.page_link (link_id, from_page_id, to_page_id, to_page_name, site_id) VALUES (34, 22, 47, NULL, 1);

INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (1, NULL, NULL, 'admin:manage', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (2, NULL, NULL, 'account:you', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (3, NULL, 'Get a new wiki', 'new-site', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (4, NULL, NULL, 'user:info', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (5, NULL, 'Side', 'nav:side', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (6, NULL, 'What Is A Wiki Site', 'what-is-a-wiki-site', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (7, NULL, 'Admin', 'profile:admin', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (8, NULL, NULL, 'admin:manage', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (10, NULL, 'Profile Side', 'nav:profile-side', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (11, NULL, 'Side', 'nav:side', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (12, NULL, NULL, 'start', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (13, NULL, 'How To Edit Pages - Quickstart', 'how-to-edit-pages', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (14, NULL, 'Join This Wiki', 'system:join', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (15, NULL, NULL, 'admin:manage', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (16, NULL, 'Page Tags List', 'system:page-tags-list', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (17, NULL, 'Recent Changes', 'system:recent-changes', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (18, NULL, 'Members', 'system:members', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (19, NULL, 'Wiki Search', 'search:site', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (20, NULL, NULL, 'system:page-tags', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (21, NULL, 'List All Pages', 'system:list-all-pages', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (22, NULL, 'Side', 'nav:side', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (23, NULL, 'Welcome to Wikidot', 'start', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (24, NULL, 'Search All Wikis', 'search:all', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (25, NULL, 'Search', 'search:site', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (26, NULL, 'Search This Wiki', 'search:site', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (27, NULL, 'Search Users', 'search:users', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (28, NULL, 'Forum Categories', 'forum:start', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (29, NULL, 'Forum Category', 'forum:category', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (30, NULL, 'Forum Thread', 'forum:thread', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (31, NULL, 'New Forum Thread', 'forum:new-thread', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (32, NULL, 'Recent Forum Posts', 'forum:recent-posts', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (33, NULL, 'Top', 'nav:top', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (34, NULL, 'Template', 'profile:template', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (37, NULL, 'Congratulations, welcome to your new wiki!', 'start', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (38, NULL, 'List of all wikis', 'system-all:all-sites', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (39, NULL, 'Sites By Tags', 'system-all:sites-by-tags', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (40, NULL, 'List wikis by tags', 'system-all:sites-by-tags', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (41, NULL, 'Search', 'system-all:search', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (42, NULL, 'Activity', 'system-all:activity', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (43, NULL, 'Activity across all wikis', 'system-all:activity', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (44, NULL, 'Welcome to your new Wikidot installation!', 'start', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (45, NULL, 'What Is A Wiki', 'what-is-a-wiki', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (46, NULL, 'How To Edit Pages', 'how-to-edit-pages', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (47, NULL, 'Wiki Members', 'system:members', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (48, NULL, 'How to join this wiki?', 'system:join', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (49, NULL, 'Recent changes', 'system:recent-changes', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (50, NULL, 'List all pages', 'system:list-all-pages', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (51, NULL, 'Page Tags List', 'system:page-tags-list', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (52, NULL, 'Page Tags', 'system:page-tags', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (53, NULL, 'Log in', 'auth:login', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (54, NULL, 'Create account - step 1', 'auth:newaccount', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (55, NULL, 'Create account - step 2', 'auth:newaccount2', 1);
INSERT INTO public.page_metadata (metadata_id, parent_page_id, title, unix_name, owner_user_id) VALUES (56, NULL, 'Create account - step 3', 'auth:newaccount3', 1);

INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (1, 1, 1, 1, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-24 12:16:34', 1, NULL, '', false, 1);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (2, 2, 2, 2, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-24 12:22:02', 1, NULL, '', false, 1);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (3, 3, 3, 3, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-24 12:27:10', 1, NULL, '', false, 1);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (4, 4, 4, 4, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-24 12:32:21', 1, NULL, '', false, 1);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (5, 5, 5, 5, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-25 00:35:20', 1, NULL, '', false, 2);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (6, 6, 6, 6, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-25 00:45:30', 1, NULL, '', false, 2);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (9, 9, 9, 9, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-25 01:08:10', 1, NULL, '', false, 1);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (14, 13, 14, 13, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-29 00:09:59', 1, NULL, '', false, 2);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (15, 14, 15, 14, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-29 00:56:59', 1, NULL, '', false, 2);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (16, 15, 16, 15, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-29 00:57:39', 1, NULL, '', false, 2);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (17, 16, 17, 16, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-29 00:58:44', 1, NULL, '', false, 2);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (18, 17, 18, 17, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-29 00:59:14', 1, NULL, '', false, 2);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (19, 18, 19, 18, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-29 00:59:40', 1, NULL, '', false, 2);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (20, 19, 20, 19, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-29 01:01:49', 1, NULL, '', false, 2);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (21, 20, 21, 20, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-29 01:03:43', 1, NULL, '', false, 2);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (22, 21, 22, 21, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-29 01:04:52', 1, NULL, '', false, 2);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (23, 22, 23, 22, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-29 01:05:47', 1, NULL, '', false, 1);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (24, 23, 24, 23, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-29 01:07:41', 1, NULL, '', false, 1);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (25, 24, 25, 24, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-29 01:09:17', 1, NULL, '', false, 1);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (26, 25, 26, 25, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-29 01:34:40', 1, NULL, '', false, 1);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (27, 25, 26, 26, NULL, false, true, false, false, false, false, 0, false, 1, '2008-01-29 01:34:57', 1, NULL, '', false, 1);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (28, 23, 27, 23, NULL, true, false, false, false, false, false, 0, false, 1, '2008-01-29 01:35:41', 1, NULL, '', false, 1);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (29, 26, 28, 27, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-29 01:36:56', 1, NULL, '', false, 1);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (30, 26, 29, 27, NULL, true, false, false, false, false, false, 0, false, 1, '2008-01-29 01:37:12', 1, NULL, '', false, 1);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (31, 27, 30, 28, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-29 01:40:23', 1, NULL, '', false, 2);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (32, 28, 31, 29, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-29 01:40:59', 1, NULL, '', false, 2);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (33, 29, 32, 30, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-29 01:41:32', 1, NULL, '', false, 2);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (34, 30, 33, 31, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-29 01:42:10', 1, NULL, '', false, 2);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (35, 31, 34, 32, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-29 01:42:42', 1, NULL, '', false, 2);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (36, 32, 35, 33, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-29 23:29:51', 1, NULL, '', false, 2);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (37, 33, 36, 34, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-29 23:30:18', 1, NULL, '', false, 2);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (38, 34, 37, 35, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-30 08:39:24', 1, NULL, '', false, 2);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (39, 35, 38, 36, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-30 08:40:31', 1, NULL, '', false, 2);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (40, 36, 39, 37, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-30 08:43:22', 1, NULL, '', false, 2);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (41, 22, 40, 22, NULL, true, false, false, false, false, false, 0, false, 1, '2008-01-30 08:53:14', 1, NULL, '', false, 1);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (42, 37, 41, 38, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-30 08:54:56', 1, NULL, '', false, 1);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (43, 38, 42, 39, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-30 08:55:33', 1, NULL, '', false, 1);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (44, 38, 43, 40, NULL, true, true, false, false, false, false, 0, false, 1, '2008-01-30 09:00:00', 1, NULL, '', false, 1);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (45, 22, 44, 22, NULL, true, false, false, false, false, false, 0, false, 2, '2008-01-30 09:01:50', 1, NULL, '', false, 1);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (46, 39, 45, 41, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-30 09:07:05', 1, NULL, '', false, 1);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (47, 40, 46, 42, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-30 09:16:38', 1, NULL, '', false, 1);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (48, 40, 47, 43, NULL, true, true, false, false, false, false, 0, false, 1, '2008-01-30 09:17:40', 1, NULL, '', false, 1);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (49, 23, 48, 44, NULL, true, true, false, false, false, false, 0, false, 2, '2008-01-30 12:52:23', 1, NULL, '', false, 1);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (50, 23, 49, 44, NULL, true, false, false, false, false, false, 0, false, 3, '2008-01-30 16:08:02', 1, NULL, '', false, 1);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (51, 41, 50, 45, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-30 16:11:56', 1, NULL, '', false, 1);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (52, 13, 51, 13, NULL, true, false, false, false, false, false, 0, false, 1, '2008-01-30 16:12:40', 1, NULL, '', false, 2);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (53, 42, 52, 46, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-30 16:12:48', 1, NULL, '', false, 1);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (54, 43, 53, 47, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-30 16:13:32', 1, NULL, '', false, 1);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (55, 44, 54, 48, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-30 16:14:13', 1, NULL, '', false, 1);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (56, 45, 55, 49, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-30 16:14:41', 1, NULL, '', false, 1);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (57, 46, 56, 50, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-30 16:15:22', 1, NULL, '', false, 1);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (58, 47, 57, 51, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-30 16:15:56', 1, NULL, '', false, 1);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (59, 48, 58, 52, NULL, false, false, false, false, false, true, 0, false, 0, '2008-01-30 16:16:22', 1, NULL, '', false, 1);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (60, 49, 59, 53, NULL, false, false, false, false, false, true, 0, false, 0, '2008-08-19 16:25:58', 1, NULL, NULL, false, 1);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (61, 50, 60, 54, NULL, false, false, false, false, false, true, 0, false, 0, '2008-08-19 16:25:58', 1, NULL, NULL, false, 1);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (62, 51, 61, 55, NULL, false, false, false, false, false, true, 0, false, 0, '2008-08-19 16:25:58', 1, NULL, NULL, false, 1);
INSERT INTO public.page_revision (revision_id, page_id, source_id, metadata_id, flags, flag_text, flag_title, flag_file, flag_rename, flag_meta, flag_new, since_full_source, diff_source, revision_number, date_last_edited, user_id, user_string, comments, flag_new_site, site_id) VALUES (63, 52, 62, 56, NULL, false, false, false, false, false, true, 0, false, 0, '2008-08-19 16:25:58', 1, NULL, NULL, false, 1);

INSERT INTO public.page_source (source_id, text) VALUES (1, '[[module ManageSite]]');
INSERT INTO public.page_source (source_id, text) VALUES (2, '[[module Account]]');
INSERT INTO public.page_source (source_id, text) VALUES (3, 'Use this simple form to create a new wiki.

To admins: you can customize this page by simply clicking "edit" at the bottom of the page.

[[module NewSite]]');
INSERT INTO public.page_source (source_id, text) VALUES (4, '[[module UserInfo]]');
INSERT INTO public.page_source (source_id, text) VALUES (5, '* [[[start | Welcome page]]]

* [[[What is a Wiki Site?]]]
* [[[How to edit pages?]]]

* [[[system: join | How to join this site?]]]
* [[[system:members | Site members]]]

* [[[system: Recent changes]]]
* [[[system: List all pages]]]
* [[[system:page-tags-list|Page Tags]]]

* [[[admin:manage|Site Manager]]]

++ Page tags
[[module TagCloud minFontSize="80%" maxFontSize="200%"  maxColor="8,8,64" minColor="100,100,128" target="system:page-tags" limit="30"]]

++ Add a new page
[[module NewPage size="15" button="new page"]]

= [[size 80%]][[[nav:side | edit this panel]]][[/size]]');
INSERT INTO public.page_source (source_id, text) VALUES (6, 'According to [http://en.wikipedia.org/wiki/Wiki Wikipedia], the world largest wiki site:

> A //Wiki// ([ˈwiː.kiː] <wee-kee> or [ˈwɪ.kiː] <wick-ey>) is a type of website that allows users to add, remove, or otherwise edit and change most content very quickly and easily.

And that is it! As a part of a farm of wikis this site is a great tool that you can use to publish content, upload files, communicate and collaborate.');
INSERT INTO public.page_source (source_id, text) VALUES (8, '[[module ManageSite]]');
INSERT INTO public.page_source (source_id, text) VALUES (10, 'The profiles site is used to host user profiles. Each {{profile:username}} page contains a user-editable text that is included in the user''s profile page.

If you are viewing your own profile content page, feel free to edit it. You are the only one allowed to edit this page.');
INSERT INTO public.page_source (source_id, text) VALUES (11, '* [[[start | Main page]]]
* [[[admin:manage | Manage this wiki]]]');
INSERT INTO public.page_source (source_id, text) VALUES (12, 'The profiles site is used to host user profiles. Each {{profile:username}} page contains a user-editable text that is included in the user''s profile page.

* [[[start | Main page]]]
* [[[admin:manage | Manage this wiki]]]');
INSERT INTO public.page_source (source_id, text) VALUES (13, 'The purpose of this wiki is to store user profiles.');
INSERT INTO public.page_source (source_id, text) VALUES (14, 'If you are allowed to edit pages in this Site, simply click on //edit// button at the bottom of the page. This will open an editor with a toolbar pallette with options.

To create a link to a new page, use syntax: {{``[[[new page name]]]``}} or {{``[[[new page name | text to display]]]``}}. Follow the link (which should have a different color if page does not exist) and create a new page and edit it!

Although creating and editing pages is easy, there are a lot more options that allows creating powerful sites. Please visit [*http://www.wikidot.com/doc Documentation pages] (at wikidot.com) to learn more.');
INSERT INTO public.page_source (source_id, text) VALUES (15, '[[note]]
Please change this page according to your policy (configure first using [[[admin:manage|Site Manager]]]) and remove this note.
[[/note]]

+ Who can join?

You can write here who can become a member of this site.

+ Join!

So you want to become a member of this site? Tell us why and apply now!

[[module MembershipApply]]

Or, if you already know a "secret password", go for it!

[[module MembershipByPassword]]');
INSERT INTO public.page_source (source_id, text) VALUES (16, '[[module ManageSite]]');
INSERT INTO public.page_source (source_id, text) VALUES (17, '[[module TagCloud limit="200" target="system:page-tags"]]

[!--

You can edit parameters of the TagCloud module as described in http://www.wikidot.com/doc:tagcloud-module
But if you want to keep the tag functionality working - do not remove these modules.

--]');
INSERT INTO public.page_source (source_id, text) VALUES (18, '[[module SiteChanges]]');
INSERT INTO public.page_source (source_id, text) VALUES (19, '+ Members:

[[module Members]]

+ Moderators

[[module Members group="moderators"]]

+ Admins

[[module Members group="admins"]]');
INSERT INTO public.page_source (source_id, text) VALUES (20, '[[module Search]]

[!-- please do not remove or change this page if you want to keep the search function working --]');
INSERT INTO public.page_source (source_id, text) VALUES (21, '[[div style="float:right; width: 50%;"]]
[[module TagCloud limit="200" target="system:page-tags"]]
[[/div]]
[[module PagesByTag]]

[!--

You can edit parameters of the TagCloud module as described in http://www.wikidot.com/doc:tagcloud-module
But if you want to keep the tag functionality working - do not remove these modules.

--]');
INSERT INTO public.page_source (source_id, text) VALUES (22, '[[module Pages preview="true"]]');
INSERT INTO public.page_source (source_id, text) VALUES (23, '* [[[start | Welcome page]]]

* [[[What is a Wiki Site?]]]
* [[[How to edit pages?]]]

* [[[system: join | How to join this site?]]]
* [[[system:members | Site members]]]

* [[[system: Recent changes]]]
* [[[system: List all pages]]]
* [[[system:page-tags-list|Page Tags]]]

* [[[admin:manage|Site Manager]]]

++ Page tags
[[module TagCloud minFontSize="80%" maxFontSize="200%"  maxColor="8,8,64" minColor="100,100,128" target="system:page-tags" limit="30"]]

++ Add a new page
[[module NewPage size="15" button="new page"]]

= [[size 80%]][[[nav:side | edit this panel]]][[/size]]');
INSERT INTO public.page_source (source_id, text) VALUES (24, 'Welcome to your new Wikidot installation.

+ Search all wikis

[[module SearchAll]]');
INSERT INTO public.page_source (source_id, text) VALUES (25, '[[module SearchAll]]');
INSERT INTO public.page_source (source_id, text) VALUES (26, '[[module Search]]');
INSERT INTO public.page_source (source_id, text) VALUES (27, 'Welcome to your new Wikidot installation.

+ Search all wikis

[[module SearchAll]]

+ Search users

[[module SearchUsers]]');
INSERT INTO public.page_source (source_id, text) VALUES (28, 'To look for someone, please enter:

* email address of a person you are looking for (this will look for exact match)
* any part of the screen name or realname (lists all Users matching the query)

[[module UserSearch]]');
INSERT INTO public.page_source (source_id, text) VALUES (29, 'To look for someone, please enter:

* email address of a person you are looking for (this will look for exact match)
* any part of the screen name or realname (lists all Users matching the query)

[[module SearchUsers]]');
INSERT INTO public.page_source (source_id, text) VALUES (30, '[[module ForumStart]]
[!-- please do not alter this page if you want to keep your forum working --]');
INSERT INTO public.page_source (source_id, text) VALUES (31, '[[module ForumCategory]]

[!-- please do not alter this page if you want to keep your forum working --]');
INSERT INTO public.page_source (source_id, text) VALUES (32, '[[module ForumThread]]

[!-- please do not alter this page if you want to keep your forum working --]');
INSERT INTO public.page_source (source_id, text) VALUES (33, '[[module ForumNewThread]]

[!-- please do not alter this page if you want to keep your forum working --]');
INSERT INTO public.page_source (source_id, text) VALUES (34, '[[module RecentPosts]]

[!-- please do not alter this page if you want to keep your forum working --]');
INSERT INTO public.page_source (source_id, text) VALUES (35, '* [# example menu]
 * [[[submenu]]]
* [[[contact]]]

[!-- top nav menu, use only one bulleted list above --]');
INSERT INTO public.page_source (source_id, text) VALUES (36, 'Profile has not been created (yet).');
INSERT INTO public.page_source (source_id, text) VALUES (50, 'According to [http://en.wikipedia.org/wiki/Wiki Wikipedia], the world largest wiki site:

> A //Wiki// ([ˈwiː.kiː] <wee-kee> or [ˈwɪ.kiː] <wick-ey>) is a type of website that allows users to add, remove, or otherwise edit and change most content very quickly and easily.

And that is it! As a part of a farm of wikis this site is a great tool that you can use to publish content, upload files, communicate and collaborate.');
INSERT INTO public.page_source (source_id, text) VALUES (39, '++ If this is your first site

Then there are some things you need to know:

* You can configure all security and other settings online, using the [[[admin:manage | Site Manager]]].  When you invite other people to help build this site they don''t have access to the Site Manager unless you make them administrators like yourself.  Check out the //Permissions// section.
* Your Wikidot site has two menus, [[[nav:side | one at the side]]] called ''{{nav:side}}'', and [[[nav:top | one at the top]]] called ''{{nav:top}}''.  These are Wikidot pages, and you can edit them like any page.
* To edit a page, go to the page and click the **Edit** button at the bottom.  You can change everything in the main area of your page.  The Wikidot system is [*http://www.wikidot.org/doc easy to learn and powerful].
* You can attach images and other files to any page, then display them and link to them in the page.
* Every Wikidot page has a history of edits, and you can undo anything.  So feel secure, and experiment.
* To start a forum on your site, see the [[[admin:manage | Site Manager]]] >> //Forum//.
* The license for this Wikidot site has been set to [*http://creativecommons.org/licenses/by-sa/3.0/ Creative Commons Attribution-Share Alike 3.0 License].  If you want to change this, use the Site Manager.
* If you want to learn more, make sure you visit the [*http://www.wikidot.org/doc Documentation section at www.wikidot.org]

More information about the Wikidot project can be found at [*http://www.wikidot.org www.wikidot.org].');
INSERT INTO public.page_source (source_id, text) VALUES (40, '* [[[start | Welcome page]]]

* [[[What is a Wiki?]]]
* [[[How to edit pages?]]]

+ All wikis

* [[[system-all:activity | Recent activity]]]
* [[[system-all:all-sites | All wikis]]]
* [[[system-all:sites-by-tags]]]
* [[[system-all:search]]]

+ This wiki

* [[[system: join | How to join this site?]]]
* [[[system:members | Site members]]]

* [[[system: Recent changes]]]
* [[[system: List all pages]]]
* [[[system:page-tags-list|Page Tags]]]

* [[[admin:manage|Site Manager]]]

++ Page tags
[[module TagCloud minFontSize="80%" maxFontSize="200%"  maxColor="8,8,64" minColor="100,100,128" target="system:page-tags" limit="30"]]

++ Add a new page
[[module NewPage size="15" button="new page"]]

= [[size 80%]][[[nav:side | edit this panel]]][[/size]]');
INSERT INTO public.page_source (source_id, text) VALUES (41, 'Below is the list of public visible Wikis hosted at this service:

[[module ListAllWikis]]');
INSERT INTO public.page_source (source_id, text) VALUES (42, '[[module SitesTagCloud limit=100]]


[[module SitesListByTag]]');
INSERT INTO public.page_source (source_id, text) VALUES (43, '[[module SitesTagCloud limit="100" target="system-all:sites-by-tags"]]


[[module SitesListByTag]]');
INSERT INTO public.page_source (source_id, text) VALUES (44, '* [[[start | Welcome page]]]

* [[[What is a Wiki?]]]
* [[[How to edit pages?]]]
* [[[new-site | Get a new wiki!]]]

+ All wikis

* [[[system-all:activity | Recent activity]]]
* [[[system-all:all-sites | All wikis]]]
* [[[system-all:sites-by-tags | Wikis by tags]]]
* [[[system-all:search | Search]]]

+ This wiki

* [[[system: join | How to join this site?]]]
* [[[system:members | Site members]]]

* [[[system: Recent changes]]]
* [[[system: List all pages]]]
* [[[system:page-tags-list|Page Tags]]]

* [[[admin:manage|Site Manager]]]

++ Page tags
[[module TagCloud minFontSize="80%" maxFontSize="200%"  maxColor="8,8,64" minColor="100,100,128" target="system:page-tags" limit="30"]]

++ Add a new page
[[module NewPage size="15" button="new page"]]

= [[size 80%]][[[nav:side | edit this panel]]][[/size]]');
INSERT INTO public.page_source (source_id, text) VALUES (45, '[[=]]
+ Search all Wikis

Perform a search through all public and visible wikis.

[[module SearchAll]]

---------------

+ Search users

To look for someone, please enter:

* email address of a person you are looking for (this will look for exact match)
* any part of the screen name or realname (lists all Users matching the query)

[[module SearchUsers]]

[[/=]]');
INSERT INTO public.page_source (source_id, text) VALUES (46, '[[table]]
[[row]]
[[cell style="width: 45%; padding-right: 2%; border-right: 1px solid #999;"]]

++ Recent edits (all wikis)

[[module RecentWRevisions]]

[[/cell]]
[[cell style="width: 45%; padding-left: 2%;"]]

++ Top Sites

[[module MostActiveSites]]

++ Top Forums

[[module MostActiveForums]]

++ New users

[[module NewWUsers]]

++ Some statistics

[[module SomeGlobalStats]]

[[/cell]]
[[/row]]
[[/table]]');
INSERT INTO public.page_source (source_id, text) VALUES (47, '[[table]]
[[row]]
[[cell style="width: 45%; padding-right: 2%; border-right: 1px solid #999; vertical-align:top;"]]

++ Recent edits (all wikis)

[[module RecentWRevisions]]

[[/cell]]
[[cell style="width: 45%; padding-left: 2%; vertical-align:top;"]]

++ Top Sites

[[module MostActiveSites]]

++ Top Forums

[[module MostActiveForums]]

++ New users

[[module NewWUsers]]

++ Some statistics

[[module SomeGlobalStats]]

[[/cell]]
[[/row]]
[[/table]]');
INSERT INTO public.page_source (source_id, text) VALUES (48, 'Congratulations, you have successfully installed Wikidot software on your computer!

+ What to do next

++ Customize this wiki

Wikidot consists of several wiki sites, not just one. Right now you are on the main wiki. Customize it!

* You can configure all security and other settings online, using the [[[admin:manage | Site Manager]]].  When you invite other people to help build this site they don''t have access to the Site Manager unless you make them administrators like yourself.  Check out the //Permissions// section.
* Your Wikidot site has two menus, [[[nav:side | one at the side]]] called ''{{nav:side}}'', and [[[nav:top | one at the top]]] called ''{{nav:top}}''.  These are Wikidot pages, and you can edit them like any page.
* To edit a page, go to the page and click the **Edit** button at the bottom.  You can change everything in the main area of your page.  The Wikidot system is [*http://www.wikidot.org/doc easy to learn and powerful].
* You can attach images and other files to any page, then display them and link to them in the page.
* Every Wikidot page has a history of edits, and you can undo anything.  So feel secure, and experiment.
* To start a forum on your site, see the [[[admin:manage | Site Manager]]] >> //Forum//.
* The license for this Wikidot site has been set to [*http://creativecommons.org/licenses/by-sa/3.0/ Creative Commons Attribution-Share Alike 3.0 License].  If you want to change this, use the Site Manager.
* If you want to learn more, make sure you visit the [*http://www.wikidot.org/doc Documentation section at www.wikidot.org]

++ Customize default template

Default initial template for other wikis is located at [[[template-en::]]]. If someone creates a new wiki, this one is cloned to the new address. A good thing to do is to go to [[[template-en::]]] and customize it.

++ Create more templates

Simply create wikis with unix names starting with "template-" (e.g. "template-pl", "template-blog") and your users will be able to choose which wiki they want to start with.

---------------

More information about the Wikidot project can be found at [*http://www.wikidot.org www.wikidot.org].

+ Search all wikis

[[module SearchAll]]

+ Search users

[[module SearchUsers]]');
INSERT INTO public.page_source (source_id, text) VALUES (49, 'Congratulations, you have successfully installed Wikidot software on your computer!

+ What to do next

++ Customize this wiki

Wikidot consists of several wiki sites, not just one. Right now you are on the main wiki. Customize it!

* You can configure all security and other settings online, using the [[[admin:manage | Site Manager]]].  When you invite other people to help build this site they don''t have access to the Site Manager unless you make them administrators like yourself.  Check out the //Permissions// section.
* Your Wikidot site has two menus, [[[nav:side | one at the side]]] called ''{{nav:side}}'', and [[[nav:top | one at the top]]] called ''{{nav:top}}''.  These are Wikidot pages, and you can edit them like any page.
* To edit a page, go to the page and click the **Edit** button at the bottom.  You can change everything in the main area of your page.  The Wikidot system is [*http://www.wikidot.org/doc easy to learn and powerful].
* You can attach images and other files to any page, then display them and link to them in the page.
* Every Wikidot page has a history of edits, and you can undo anything.  So feel secure, and experiment.
* To start a forum on your site, see the [[[admin:manage | Site Manager]]] >> //Forum//.
* The license for this Wikidot site has been set to [*http://creativecommons.org/licenses/by-sa/3.0/ Creative Commons Attribution-Share Alike 3.0 License].  If you want to change this, use the Site Manager.
* If you want to learn more, make sure you visit the [*http://www.wikidot.org/doc Documentation section at www.wikidot.org]

++ Customize default template

Default initial template for other wikis is located at [[[template-en::]]]. If someone creates a new wiki, this one is cloned to the new address. A good thing to do is to go to [[[template-en::]]] and customize it.

++ Create more templates

Simply create wikis with unix names starting with "template-" (e.g. "template-pl", "template-blog") and your users will be able to choose which wiki they want to start with.

++ Visit Wikidot.org

Go to **[http://www.wikidot.org www.wikidot.org]** -- home of the Wikidot software -- for extra documentation, howtos, tips and support.

---------------

More information about the Wikidot project can be found at [*http://www.wikidot.org www.wikidot.org].

+ Search all wikis

[[module SearchAll]]

+ Search users

[[module SearchUsers]]');
INSERT INTO public.page_source (source_id, text) VALUES (51, 'If you are allowed to edit pages in this Site, simply click on //edit// button at the bottom of the page. This will open an editor with a toolbar pallette with options.

To create a link to a new page, use syntax: {{``[[[new page name]]]``}} or {{``[[[new page name | text to display]]]``}}. Follow the link (which should have a different color if page does not exist) and create a new page and edit it!

Although creating and editing pages is easy, there are a lot more options that allows creating powerful sites. Please visit [*http://www.wikidot.org/doc Documentation pages] (at wikidot.org) to learn more.');
INSERT INTO public.page_source (source_id, text) VALUES (52, 'If you are allowed to edit pages in this Site, simply click on //edit// button at the bottom of the page. This will open an editor with a toolbar pallette with options.

To create a link to a new page, use syntax: {{``[[[new page name]]]``}} or {{``[[[new page name | text to display]]]``}}. Follow the link (which should have a different color if page does not exist) and create a new page and edit it!

Although creating and editing pages is easy, there are a lot more options that allows creating powerful sites. Please visit [*http://www.wikidot.org/doc Documentation pages] (at wikidot.org) to learn more.');
INSERT INTO public.page_source (source_id, text) VALUES (53, '+ Members:

[[module Members]]

+ Moderators

[[module Members group="moderators"]]

+ Admins

[[module Members group="admins"]]');
INSERT INTO public.page_source (source_id, text) VALUES (54, '[[note]]
Please change this page according to your policy (configure first using [[[admin:manage|Site Manager]]]) and remove this note.
[[/note]]

+ Who can join?

You can write here who can become a member of this site.

+ Join!

So you want to become a member of this site? Tell us why and apply now!

[[module MembershipApply]]

Or, if you already know a "secret password", go for it!

[[module MembershipByPassword]]');
INSERT INTO public.page_source (source_id, text) VALUES (55, '[[module SiteChanges]]');
INSERT INTO public.page_source (source_id, text) VALUES (56, '[[module Pages preview="true"]]');
INSERT INTO public.page_source (source_id, text) VALUES (57, '[[module TagCloud limit="200" target="system:page-tags"]]

[!--

You can edit parameters of the TagCloud module as described in http://www.wikidot.com/doc:tagcloud-module
But if you want to keep the tag functionality working - do not remove these modules.

--]');
INSERT INTO public.page_source (source_id, text) VALUES (58, '[[div style="float:right; width: 50%;"]]
[[module TagCloud limit="200" target="system:page-tags"]]
[[/div]]
[[module PagesByTag]]

[!--

You can edit parameters of the TagCloud module as described in http://www.wikidot.com/doc:tagcloud-module
But if you want to keep the tag functionality working - do not remove these modules.

--]');
INSERT INTO public.page_source (source_id, text) VALUES (59, '[[module LoginModule]]');
INSERT INTO public.page_source (source_id, text) VALUES (60, '[[module CreateAccountStep1]]');
INSERT INTO public.page_source (source_id, text) VALUES (61, '[[module CreateAccountStep2]]');
INSERT INTO public.page_source (source_id, text) VALUES (62, '[[module CreateAccountStep3]]');

INSERT INTO public.profile (user_id, real_name, pronouns, birthday_day, birthday_month, birthday_year, about, location, website, im_icq, im_jabber, change_screen_name_count) VALUES (1, NULL, NULL, NULL, NULL, NULL, 'Wikidot administrator.', NULL, NULL, NULL, NULL, 0);

INSERT INTO public.site (site_id, name, subtitle, unix_name, description, language, date_created, custom_domain, visible, default_page, private, deleted) VALUES (1, 'Wikijump', 'Fighting Ozone Pollution', 'www', 'Wikijump host site', 'en', NULL, NULL, true, 'start', false, false);
INSERT INTO public.site (site_id, name, subtitle, unix_name, description, language, date_created, custom_domain, visible, default_page, private, deleted) VALUES (2, 'Template site (en)', 'Default template wiki', 'template-en', '', 'en', NULL, NULL, true, 'start', false, false);

INSERT INTO public.site_settings (site_id, allow_membership_by_apply, allow_membership_by_password, membership_password, file_storage_size, use_ganalytics, private_landing_page, max_private_members, max_private_viewers, hide_navigation_unauthorized, ssl_mode, allow_members_invite, max_upload_file_size) VALUES (1, true, false, NULL, 1073741824, false, 'system:join', 50, 20, true, NULL, false, 10485760);
INSERT INTO public.site_settings (site_id, allow_membership_by_apply, allow_membership_by_password, membership_password, file_storage_size, use_ganalytics, private_landing_page, max_private_members, max_private_viewers, hide_navigation_unauthorized, ssl_mode, allow_members_invite, max_upload_file_size) VALUES (2, false, false, '', 314572800, false, 'system:join', 50, 20, true, NULL, false, 10485760);

INSERT INTO public.site_super_settings (site_id, can_custom_domain) VALUES (1, true);
INSERT INTO public.site_super_settings (site_id, can_custom_domain) VALUES (2, true);

INSERT INTO public.site_tag (tag_id, site_id, tag) VALUES (1, 2, 'template');

INSERT INTO public.theme (theme_id, name, unix_name, abstract, extends_theme_id, variant_of_theme_id, custom, site_id, use_side_bar, use_top_bar, sort_index, sync_page_name, revision_number) VALUES (1, 'Base', 'base', true, NULL, NULL, false, NULL, true, true, 0, NULL, 0);
INSERT INTO public.theme (theme_id, name, unix_name, abstract, extends_theme_id, variant_of_theme_id, custom, site_id, use_side_bar, use_top_bar, sort_index, sync_page_name, revision_number) VALUES (2, 'Clean', 'clean', false, 1, NULL, false, NULL, true, true, 0, NULL, 0);
INSERT INTO public.theme (theme_id, name, unix_name, abstract, extends_theme_id, variant_of_theme_id, custom, site_id, use_side_bar, use_top_bar, sort_index, sync_page_name, revision_number) VALUES (4, 'Flannel', 'flannel', false, 1, NULL, false, NULL, true, true, 0, NULL, 0);
INSERT INTO public.theme (theme_id, name, unix_name, abstract, extends_theme_id, variant_of_theme_id, custom, site_id, use_side_bar, use_top_bar, sort_index, sync_page_name, revision_number) VALUES (6, 'Flannel Ocean', 'flannel-ocean', false, 1, NULL, false, NULL, true, true, 0, NULL, 0);
INSERT INTO public.theme (theme_id, name, unix_name, abstract, extends_theme_id, variant_of_theme_id, custom, site_id, use_side_bar, use_top_bar, sort_index, sync_page_name, revision_number) VALUES (8, 'Flannel Nature', 'flannel-nature', false, 1, NULL, false, NULL, true, true, 0, NULL, 0);
INSERT INTO public.theme (theme_id, name, unix_name, abstract, extends_theme_id, variant_of_theme_id, custom, site_id, use_side_bar, use_top_bar, sort_index, sync_page_name, revision_number) VALUES (10, 'Cappuccino', 'cappuccino', false, 1, NULL, false, NULL, true, true, 0, NULL, 0);
INSERT INTO public.theme (theme_id, name, unix_name, abstract, extends_theme_id, variant_of_theme_id, custom, site_id, use_side_bar, use_top_bar, sort_index, sync_page_name, revision_number) VALUES (12, 'Gila', 'gila', false, 1, NULL, false, NULL, true, true, 0, NULL, 0);
INSERT INTO public.theme (theme_id, name, unix_name, abstract, extends_theme_id, variant_of_theme_id, custom, site_id, use_side_bar, use_top_bar, sort_index, sync_page_name, revision_number) VALUES (14, 'Co', 'co', false, 1, NULL, false, NULL, true, true, 0, NULL, 0);
INSERT INTO public.theme (theme_id, name, unix_name, abstract, extends_theme_id, variant_of_theme_id, custom, site_id, use_side_bar, use_top_bar, sort_index, sync_page_name, revision_number) VALUES (15, 'Flower Blossom', 'flower-blossom', false, 1, NULL, false, NULL, true, true, 0, NULL, 0);
INSERT INTO public.theme (theme_id, name, unix_name, abstract, extends_theme_id, variant_of_theme_id, custom, site_id, use_side_bar, use_top_bar, sort_index, sync_page_name, revision_number) VALUES (16, 'Localize', 'localize', false, 1, NULL, false, NULL, true, true, 0, NULL, 0);
INSERT INTO public.theme (theme_id, name, unix_name, abstract, extends_theme_id, variant_of_theme_id, custom, site_id, use_side_bar, use_top_bar, sort_index, sync_page_name, revision_number) VALUES (20, 'Webbish', 'webbish2', false, 1, NULL, false, NULL, true, true, 0, NULL, 0);
INSERT INTO public.theme (theme_id, name, unix_name, abstract, extends_theme_id, variant_of_theme_id, custom, site_id, use_side_bar, use_top_bar, sort_index, sync_page_name, revision_number) VALUES (3, 'Clean - no side bar', 'clean-no-side-bar', false, 2, 2, false, NULL, false, true, 0, NULL, 0);
INSERT INTO public.theme (theme_id, name, unix_name, abstract, extends_theme_id, variant_of_theme_id, custom, site_id, use_side_bar, use_top_bar, sort_index, sync_page_name, revision_number) VALUES (5, 'Flannel - no side bar', 'flannel-no-side-bar', false, 4, 4, false, NULL, false, true, 0, NULL, 0);
INSERT INTO public.theme (theme_id, name, unix_name, abstract, extends_theme_id, variant_of_theme_id, custom, site_id, use_side_bar, use_top_bar, sort_index, sync_page_name, revision_number) VALUES (7, 'Flannel Ocean - no side bar', 'flannel-ocean-no-side-bar', false, 6, 6, false, NULL, false, true, 0, NULL, 0);
INSERT INTO public.theme (theme_id, name, unix_name, abstract, extends_theme_id, variant_of_theme_id, custom, site_id, use_side_bar, use_top_bar, sort_index, sync_page_name, revision_number) VALUES (9, 'Flannel Nature - no side bar', 'flannel-nature-no-side-bar', false, 8, 8, false, NULL, false, true, 0, NULL, 0);
INSERT INTO public.theme (theme_id, name, unix_name, abstract, extends_theme_id, variant_of_theme_id, custom, site_id, use_side_bar, use_top_bar, sort_index, sync_page_name, revision_number) VALUES (11, 'Cappuccino - no side bar', 'cappuccino-no-side-bar', false, 10, 10, false, NULL, false, true, 0, NULL, 0);
INSERT INTO public.theme (theme_id, name, unix_name, abstract, extends_theme_id, variant_of_theme_id, custom, site_id, use_side_bar, use_top_bar, sort_index, sync_page_name, revision_number) VALUES (13, 'Gila - no side bar', 'gila-no-side-bar', false, 12, 12, false, NULL, false, true, 0, NULL, 0);
INSERT INTO public.theme (theme_id, name, unix_name, abstract, extends_theme_id, variant_of_theme_id, custom, site_id, use_side_bar, use_top_bar, sort_index, sync_page_name, revision_number) VALUES (17, 'Localize - no side bar', 'localize-no-side-bar', false, 16, 16, false, NULL, false, true, 0, NULL, 0);
INSERT INTO public.theme (theme_id, name, unix_name, abstract, extends_theme_id, variant_of_theme_id, custom, site_id, use_side_bar, use_top_bar, sort_index, sync_page_name, revision_number) VALUES (18, 'Flower Blossom - no side bar', 'flower-blossom-no-side-bar', false, 15, 15, false, NULL, false, true, 0, NULL, 0);
INSERT INTO public.theme (theme_id, name, unix_name, abstract, extends_theme_id, variant_of_theme_id, custom, site_id, use_side_bar, use_top_bar, sort_index, sync_page_name, revision_number) VALUES (19, 'Co - no side bar', 'co-no-side-bar', false, 14, 14, false, NULL, false, true, 0, NULL, 0);
INSERT INTO public.theme (theme_id, name, unix_name, abstract, extends_theme_id, variant_of_theme_id, custom, site_id, use_side_bar, use_top_bar, sort_index, sync_page_name, revision_number) VALUES (21, 'Webbish - no side bar', 'webbish2-no-side-bar', false, 20, 20, false, NULL, false, true, 0, NULL, 0);
INSERT INTO public.theme (theme_id, name, unix_name, abstract, extends_theme_id, variant_of_theme_id, custom, site_id, use_side_bar, use_top_bar, sort_index, sync_page_name, revision_number) VALUES (22, 'Shiny', 'shiny', false, 1, NULL, false, NULL, true, true, 0, NULL, 0);
INSERT INTO public.theme (theme_id, name, unix_name, abstract, extends_theme_id, variant_of_theme_id, custom, site_id, use_side_bar, use_top_bar, sort_index, sync_page_name, revision_number) VALUES (23, 'Shiny - no side bar', 'shiny-no-side-bar', false, 22, 22, false, NULL, false, true, 0, NULL, 0);
INSERT INTO public.theme (theme_id, name, unix_name, abstract, extends_theme_id, variant_of_theme_id, custom, site_id, use_side_bar, use_top_bar, sort_index, sync_page_name, revision_number) VALUES (24, 'Bloo', 'bloo', false, 1, NULL, false, NULL, true, true, 0, NULL, 0);
INSERT INTO public.theme (theme_id, name, unix_name, abstract, extends_theme_id, variant_of_theme_id, custom, site_id, use_side_bar, use_top_bar, sort_index, sync_page_name, revision_number) VALUES (25, 'Bloo - no side bar', 'bloo-no-side-bar', false, 24, 24, false, NULL, false, true, 0, NULL, 0);
INSERT INTO public.theme (theme_id, name, unix_name, abstract, extends_theme_id, variant_of_theme_id, custom, site_id, use_side_bar, use_top_bar, sort_index, sync_page_name, revision_number) VALUES (26, 'Basic', 'basic', false, 1, NULL, false, NULL, true, true, 0, NULL, 0);
INSERT INTO public.theme (theme_id, name, unix_name, abstract, extends_theme_id, variant_of_theme_id, custom, site_id, use_side_bar, use_top_bar, sort_index, sync_page_name, revision_number) VALUES (28, 'Black Highlighter', 'bhl', false, 1, NULL, true, 1, true, true, 0, '', 0);
INSERT INTO public.theme (theme_id, name, unix_name, abstract, extends_theme_id, variant_of_theme_id, custom, site_id, use_side_bar, use_top_bar, sort_index, sync_page_name, revision_number) VALUES (27, 'Sigma-9', 'sigma9', false, 1, NULL, true, 1, true, true, 0, '', 0);

INSERT INTO public.user_karma (user_id, points, level) VALUES (1, 110, 2);

INSERT INTO public.user_settings (user_id, receive_invitations, receive_pm, notify_online, notify_feed, notify_email, receive_newsletter, receive_digest, allow_site_newsletters_default, max_sites_admin) VALUES (1, true, 'a    ', '*', '*', NULL, true, true, true, 3);
STATEMENT

        );
    }
}
