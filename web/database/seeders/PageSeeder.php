<?php
declare(strict_types=1);

namespace Database\Seeders;

use Illuminate\Database\Seeder;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Hash;

/**
 * Seeder of page-related tables.
 * @package Database\Seeders
 */
class PageSeeder extends Seeder
{
    const TIMESTAMP = 'Fri Jan 18 15:32:15 2019 -0700';

    // Because it's temporarily JSON, not TEXT[] yet
    const ALL_CHANGES = '["wikitext", "title", "alt_title", "slug", "tags", "metadata"]';

    /**
     * Run the database seeds.
     * Throws Exception if random_bytes() can't collect enough entropy.
     *
     * @return void
     * @throws Exception
     */
    public function run(): void
    {
        // prettier-ignore
        $www_start_wikitext_hash = $this->addString(<<<EOF
Congratulations, you have successfully deployed an instance of Wikijump!

++ Developer Information

Ensure your [https://github.com/scpwiki/wikijump/pulls pull request] has the issue in the title, and describes the changes you make. The issue should link to the PR and be in the appropriate state for your work.

* [https://github.com/scpwiki/wikijump GitHub]
* [https://github.com/scpwiki/wikijump/tree/legacy Legacy Branch]
* [https://scuttle.atlassian.net/browse/WJ Jira]
* [https://scuttle.atlassian.net/wiki/spaces/WD/overview Confluence]

If you have any issues, feel free to ask in the Wikijump chat.

++ Site Information

* You can configure all security and other settings online, using the [[[admin:manage | Site Manager]]].  When you invite other people to help build this site they don't have access to the Site Manager unless they are also administrators.  Check out the //Permissions// section.
* Your Wikidot site has two menus, [[[nav:side | the side bar]]] called '{{nav:side}}', and [[[nav:top | the top bar]]] called '{{nav:top}}'. These are Wikijump pages, and you can edit them like any other page.
* To edit a page, go to the page and click the **Edit** button at the bottom. You can change everything in the main area of your page.
* You can upload images and other files to any page, then display them and link to them in the page.
* Every Wikijump page has a history of edits, and you can undo anything.
* To start a forum on your site, see the [[[admin:manage | Site Manager]]] >> //Forum//.
* The license for this Wikidot site has been set to [*https://creativecommons.org/licenses/by-sa/4.0/ Creative Commons Attribution-Share Alike 4.0 License]. If you want to change this, use the Site Manager.

++ Wikijump

Wikijump is a fork of [*https://www.wikidot.com/ Wikidot]. More information about the Wikijump Project is available here:

* [https://wikijump.org Blog]
* [https://github.com/scpwiki/wikijump GitHub]
* [https://scuttle.atlassian.net/browse/WJ Jira]
* [https://scpwiki.com/forum/c-3335628/general-information General Information Forum (EN)]
* [https://scpwiki.com/forum/c-3335630/feature-requests Feature Requests Forum (EN)]
EOF
        );

        // prettier-ignore
        $www_start_compiled_hash = $this->addString(<<<EOF
p>Congratulations, you have successfully deployed an instance of Wikijump!</p>
<h2 id="toc0"><span>Developer Information</span></h2>
<p>Ensure your <a href="https://github.com/scpwiki/wikijump/pulls" target="_blank">pull request</a> has the issue in the title, and describes the changes you make. The issue should link to the PR and be in the appropriate state for your work.</p>
<ul>
<li><a href="https://github.com/scpwiki/wikijump" target="_blank">GitHub</a></li>
<li><a href="https://github.com/scpwiki/wikijump/tree/legacy" target="_blank">Legacy Branch</a></li>
<li><a href="https://scuttle.atlassian.net/browse/WJ" target="_blank">Jira</a></li>
<li><a href="https://scuttle.atlassian.net/wiki/spaces/WD/overview" target="_blank">Confluence</a></li>
</ul>
<p>If you have any issues, feel free to ask in the Wikijump chat.</p>
<h2 id="toc1"><span>Site Information</span></h2>
<ul>
<li>You can configure all security and other settings online, using the <a href="/admin:manage" target="_blank">Site Manager</a>. When you invite other people to help build this site they don't have access to the Site Manager unless they are also administrators. Check out the <em>Permissions</em> section.</li>
<li>Your Wikidot site has two menus, <a href="/nav:side" target="_blank">the side bar</a> called '<tt>nav:side</tt>', and <a href="/nav:top" target="_blank">the top bar</a> called '<tt>nav:top</tt>'. These are Wikijump pages, and you can edit them like any other page.</li>
<li>To edit a page, go to the page and click the <strong>Edit</strong> button at the bottom. You can change everything in the main area of your page.</li>
<li>You can upload images and other files to any page, then display them and link to them in the page.</li>
<li>Every Wikijump page has a history of edits, and you can undo anything.</li>
<li>To start a forum on your site, see the <a href="/admin:manage" target="_blank">Site Manager</a> » <em>Forum</em>.</li>
<li>The license for this Wikidot site has been set to <a href="https://creativecommons.org/licenses/by-sa/4.0/" target="_blank">Creative Commons Attribution-Share Alike 4.0 License</a>. If you want to change this, use the Site Manager.</li>
</ul>
<h2 id="toc2"><span>Wikijump</span></h2>
<p>Wikijump is a fork of <a href="https://www.wikidot.com/" target="_blank">Wikidot</a>. More information about the Wikijump Project is available here:</p>
<ul>
<li><a href="https://wikijump.org" target="_blank">Blog</a></li>
<li><a href="https://github.com/scpwiki/wikijump" target="_blank">GitHub</a></li>
<li><a href="https://scuttle.atlassian.net/browse/WJ" target="_blank">Jira</a></li>
<li><a href="https://scpwiki.com/forum/c-3335628/general-information" target="_blank">General Information Forum (EN)</a></li>
<li><a href="https://scpwiki.com/forum/c-3335630/feature-requests" target="_blank">Feature Requests Forum (EN)</a></li>
</ul>
EOF
        );

        // prettier-ignore
        $template_start_wikitext_hash = $this->addString(<<<EOF
This is the template site for this instance, or a newly-created site based on the template.

++ Site Information

* You can configure all security and other settings online, using the [[[admin:manage | Site Manager]]].  When you invite other people to help build this site they don't have access to the Site Manager unless they are also administrators.  Check out the //Permissions// section.
* Your Wikidot site has two menus, [[[nav:side | the side bar]]] called '{{nav:side}}', and [[[nav:top | the top bar]]] called '{{nav:top}}'. These are Wikijump pages, and you can edit them like any other page.
* To edit a page, go to the page and click the **Edit** button at the bottom. You can change everything in the main area of your page.
* You can upload images and other files to any page, then display them and link to them in the page.
* Every Wikijump page has a history of edits, and you can undo anything.
* To start a forum on your site, see the [[[admin:manage | Site Manager]]] >> //Forum//.
* The license for this Wikidot site has been set to [*https://creativecommons.org/licenses/by-sa/4.0/ Creative Commons Attribution-Share Alike 4.0 License]. If you want to change this, use the Site Manager.

++ Wikijump

Wikijump is a fork of [*https://www.wikidot.com/ Wikidot]. More information about the Wikijump Project is available here:

* [https://wikijump.org Blog]
* [https://github.com/scpwiki/wikijump GitHub]
* [https://scuttle.atlassian.net/browse/WJ Jira]
* [https://scpwiki.com/forum/c-3335628/general-information General Information Forum (EN)]
* [https://scpwiki.com/forum/c-3335630/feature-requests Feature Requests Forum (EN)]
EOF
        );

        // prettier-ignore
        $template_start_compiled_hash = $this->addString(<<<EOF
<p>This is the template site for this instance, or a newly-created site based on the template.</p>
<h2 id="toc0"><span>Site Information</span></h2>
<ul>
<li>You can configure all security and other settings online, using the <a href="/admin:manage" target="_blank">Site Manager</a>. When you invite other people to help build this site they don't have access to the Site Manager unless they are also administrators. Check out the <em>Permissions</em> section.</li>
<li>Your Wikidot site has two menus, <a href="/nav:side" target="_blank">the side bar</a> called '<tt>nav:side</tt>', and <a href="/nav:top" target="_blank">the top bar</a> called '<tt>nav:top</tt>'. These are Wikijump pages, and you can edit them like any other page.</li>
<li>To edit a page, go to the page and click the <strong>Edit</strong> button at the bottom. You can change everything in the main area of your page.</li>
<li>You can upload images and other files to any page, then display them and link to them in the page.</li>
<li>Every Wikijump page has a history of edits, and you can undo anything.</li>
<li>To start a forum on your site, see the <a href="/admin:manage" target="_blank">Site Manager</a> » <em>Forum</em>.</li>
<li>The license for this Wikidot site has been set to <a href="https://creativecommons.org/licenses/by-sa/4.0/" target="_blank">Creative Commons Attribution-Share Alike 4.0 License</a>. If you want to change this, use the Site Manager.</li>
</ul>
<h2 id="toc1"><span>Wikijump</span></h2>
<p>Wikijump is a fork of <a href="https://www.wikidot.com/" target="_blank">Wikidot</a>. More information about the Wikijump Project is available here:</p>
<ul>
<li><a href="https://wikijump.org" target="_blank">Blog</a></li>
<li><a href="https://github.com/scpwiki/wikijump" target="_blank">GitHub</a></li>
<li><a href="https://scuttle.atlassian.net/browse/WJ" target="_blank">Jira</a></li>
<li><a href="https://scpwiki.com/forum/c-3335628/general-information" target="_blank">General Information Forum (EN)</a></li>
<li><a href="https://scpwiki.com/forum/c-3335630/feature-requests" target="_blank">Feature Requests Forum (EN)</a></li>
</ul>
EOF
        );

        // prettier-ignore
        $platform_activity_wikitext_hash = $this->addString(<<<EOF
[[table]]
[[row]]
[[cell style="width: 45%; padding-right: 2%; border-right: 1px solid #999; vertical-align: top;"]]
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
[[/table]]
EOF
        );

        // prettier-ignore
        $platform_activity_compiled_hash = $this->addString(<<<EOF
<table>
<tbody><tr>
<td style="width: 45%; padding-right: 2%; border-right: 1px solid #999; vertical-align: top;">
<h2 id="toc0"><span>Recent edits (all wikis)</span></h2>
<div class="error-block">[[module <em>RecentWRevisions</em>]] No such module, please <a href="http://www.wikidot.com/doc:modules" target="_blank">check available modules</a> and fix this page.</div>
</td>
<td style="width: 45%; padding-left: 2%; vertical-align:top;">
<h2 id="toc1"><span>Top Sites</span></h2>
<div class="error-block">[[module <em>MostActiveSites</em>]] No such module, please <a href="http://www.wikidot.com/doc:modules" target="_blank">check available modules</a> and fix this page.</div>
<h2 id="toc2"><span>Top Forums</span></h2>
<div class="error-block">[[module <em>MostActiveForums</em>]] No such module, please <a href="http://www.wikidot.com/doc:modules" target="_blank">check available modules</a> and fix this page.</div>
<h2 id="toc3"><span>New users</span></h2>
<div class="error-block">[[module <em>NewWUsers</em>]] No such module, please <a href="http://www.wikidot.com/doc:modules" target="_blank">check available modules</a> and fix this page.</div>
<h2 id="toc4"><span>Some statistics</span></h2>
<div class="error-block">[[module <em>SomeGlobalStats</em>]] No such module, please <a href="http://www.wikidot.com/doc:modules" target="_blank">check available modules</a> and fix this page.</div>
</td>
</tr>
</tbody></table>
EOF
        );

        // prettier-ignore
        $platform_sites_wikitext_hash = $this->addString(<<<EOF
Below is the list of public site hosted on this instance:

[[module ListAllWikis]]
EOF
        );

        // prettier-ignore
        $platform_sites_compiled_hash = $this->addString(<<<EOF
<p>Below is the list of public site hosted on this instance:</p>
<div class="error-block">[[module <em>ListAllWikis</em>]] No such module, please <a href="http://www.wikidot.com/doc:modules" target="_blank">check available modules</a> and fix this page.</div>
EOF
        );

        // prettier-ignore
        $platform_search_wikitext_hash = $this->addString(<<<EOF
[[=]]
+ Search all Wikis

Perform a search through all public and visible wikis.

[[module SearchAll]]

---------------

+ Search users

To look for someone, please enter:

* email address of a person you are looking for (this will look for exact match)
* any part of the screen name or realname (lists all Users matching the query)

[[module SearchUsers]]
[[/=]]
EOF
        );

        // prettier-ignore
        $platform_search_compiled_hash = $this->addString(<<<EOF
<div style="text-align: center;">
<h1 id="toc0"><span>Search all Wikis</span></h1>
<p>Perform a search through all public and visible wikis.</p>
<style>
@import url(/common--modules/css/Wiki/PagesTagCloud/PagesTagCloudModule.css);
@import url(/common--modules/css/Search/SearchAllModule.css);

</style><div class="search-box">

<div class="query-area">
<form action="dummy" id="search-form-all">
<div>
Search query:
<input class="text" type="text" size="30" name="query" id="search-form-all-input" value="">
<input class="button" type="submit" value="Search">
</div>
<div style="font-size: 87%; margin-top:5px;">
<input id="search-all-pf" class="radio" type="radio" name="area" value="pf" checked="checked"><label for="search-all-pf">pages and forums</label>
<input id="search-all-p" class="radio" type="radio" name="area" value="p"><label for="search-all-p">pages only</label>
<input id="search-all-f" class="radio" type="radio" name="area" value="f"><label for="search-all-f">forums only</label>
</div>
</form>
</div>

<div class="search-results">
</div>

</div>
<hr>
<h1 id="toc1"><span>Search users</span></h1>
<p>To look for someone, please enter:</p>
<ul>
<li>email address of a person you are looking for (this will look for exact match)</li>
<li>any part of the screen name or realname (lists all Users matching the query)</li>
</ul>
<style>
@import url(/common--modules/css/Wiki/PagesTagCloud/PagesTagCloudModule.css);
@import url(/common--modules/css/Search/SearchAllModule.css);
@import url(/common--modules/css/Search/UserSearchModule.css);

</style>

<div class="search-box">
<div class="query-area">
<form action="javascript:;" id="search-form-user">
<div>

<input class="text" type="text" size="30" name="query" value="">
<input class="button" type="submit" value="Search">
</div>
</form>
</div>

<div class="search-user-results">
</div>
</div>
</div>
EOF
        );

        // prettier-ignore
        $system_join_wikitext_hash = $this->addString(<<<EOF
[[note]]
Please change this page according to your policy (configure first using [[[admin:manage|Site Manager]]]) and remove this note.
[[/note]]

+ Who can join?

You can write here who can become a member of this site.

+ Join!

So you want to become a member of this site? Tell us why and apply now!

[[module MembershipApply]]

Or, if you already know a "secret password", go for it!

[[module MembershipByPassword]]
EOF
        );

        // prettier-ignore
        $system_join_compiled_hash = $this->addString(<<<EOF
<div class="wiki-note">
<p>Please change this page according to your policy (configure first using <a href="/admin:manage">Site Manager</a>) and remove this note.</p>
</div>
<h1 id="toc0"><span>Who can join?</span></h1>
<p>You can write here who can become a member of this site.</p>
<h1 id="toc1"><span>Join!</span></h1>
<p>So you want to become a member of this site? Tell us why and apply now!</p>
<style>
@import url(/common--modules/css/Wiki/PagesTagCloud/PagesTagCloudModule.css);

</style><div id="membership-apply-box">
<p>
You should have a valid Wikijump account and be logged in order to apply for membership.</p>
<table style="margin: 1em auto">
<tbody><tr>
<td style="text-align: center; padding: 1em">
<div style="font-size: 180%; font-weight: bold;">
<a href="javascript:;" onclick="Wikijump.page.listeners.loginClick(event)">log in</a>
</div>
<p>
if you already have an account at Wikijump</p>
</td>
<td style="padding: 1em; font-size: 140%">
or
</td>
<td style="text-align: center; padding: 1em">
<div style="font-size: 180%; font-weight: bold;">
<a href="javascript:;" onclick="Wikijump.page.listeners.createAccount(event)">create a new account</a>
</div>
</td>
</tr>
</tbody></table>
</div>

<p>Or, if you already know a "secret password", go for it!</p>
<style>
@import url(/common--modules/css/Wiki/PagesTagCloud/PagesTagCloudModule.css);

</style><div id="membership-by-password-box">
<div class="error-block">
You cannot apply.<br>
Membership via password is not enabled for this site.
</div>
</div>
EOF
        );

        // prettier-ignore
        $system_members_wikitext_hash = $this->addString(<<<EOF
+ Members:

[[module Members]]

+ Moderators

[[module Members group="moderators"]]

+ Admins

[[module Members group="admins"]]
EOF
        );

        // prettier-ignore
        $system_members_compiled_hash = $this->addString(<<<EOF
<h1 id="toc0"><span>Members:</span></h1>
<style>
@import url(/common--modules/css/Wiki/PagesTagCloud/PagesTagCloudModule.css);

</style>
<div>
No users.
</div>
<h1 id="toc1"><span>Moderators</span></h1>
<style>
@import url(/common--modules/css/Wiki/PagesTagCloud/PagesTagCloudModule.css);

</style>
<div>
No users.
</div>
<h1 id="toc2"><span>Admins</span></h1>
<style>
@import url(/common--modules/css/Wiki/PagesTagCloud/PagesTagCloudModule.css);

</style>
<div>
<table>
<tbody><tr>
<td><span class="printuser avatarhover"><a href="http://www.wikijump.localhost/user:info/admin" onclick="Wikijump.page.listeners.userInfo(1); return false;"><img class="small" src="https://ui-avatars.com/api/?name=a&amp;color=FFFFFF&amp;background=512da8" alt="admin" style="background-image:url(http://www.wikijump.localhost/user--karma/1)"></a><a href="http://www.wikijump.localhost/user:info/admin" onclick="Wikijump.page.listeners.userInfo(1); return false;">admin</a></span></td>
</tr>
</tbody></table>
</div>
</div>
EOF
        );

        // prettier-ignore
        $system_recent_changes_wikitext_hash = $this->addString(<<<EOF
[[module SiteChanges]]
EOF
        );

        // prettier-ignore
        $system_recent_changes_compiled_hash = $this->addString(<<<EOF
Needs recompile: SiteChanges
EOF
        );

        // prettier-ignore
        $system_page_tags_wikitext_hash = $this->addString(<<<EOF
[[module TagCloud]]
EOF
        );

        // prettier-ignore
        $system_page_tags_compiled_hash = $this->addString(<<<EOF
It seems you have no tags attached to pages. To attach a tag simply click on the tags button at the bottom of any page.
EOF
        );

        // prettier-ignore
        $admin_manage_wikitext_hash = $this->addString(<<<EOF
[[module Redirect destination="_admin"]]
EOF
        );

        // prettier-ignore
        $admin_manage_compiled_hash = $this->addString(<<<EOF
Site Manager here
EOF
        );

        // prettier-ignore
        $nav_side_wikitext_hash = $this->addString(<<<EOF
* [[[start | Homepage]]]

+ All wikis

* [[[:www:platform:activity | Recent activity]]]
* [[[:www:platform:sites | All wikis]]]
* [[[:www:platform:search | Search]]]

+ This wiki

* [[[system:join | How to join this site?]]]
* [[[system:members | Site members]]]

* [[[system: Recent changes]]]
* [[[system: Page Tags]]]

* [[[admin:manage|Site Manager]]]

++ Add a new page
[[module NewPage size="15" button="new page"]]

= [[size 80%]][[[nav:side | edit this panel]]][[/size]]
EOF
        );

        // prettier-ignore
        $nav_side_compiled_hash = $this->addString(<<<EOF
<ul><li><a href="/start">Welcome page</a></li></ul><ul><li><a href="/what-is-a-wiki">What is a Wiki?</a></li><li><a href="/how-to-edit-pages">How to edit pages?</a></li><li><a href="/new-site">Get a new wiki!</a></li></ul><h1><span>All wikis</span></h1><ul><li><a href="/platform:activity">Recent activity</a></li><li class=""><a href="/platform:all-sites">All wikis</a></li><li class=""><a href="/platform:sites-by-tags">Wikis by tags</a></li><li class=""><a href="/platform:search">Search</a></li></ul><h1><span>This wiki</span></h1><ul><li class=""><a href="/system:join">How to join this site?</a></li><li><a href="/system:members">Site members</a></li></ul><ul><li><a href="/system:recent-changes">Recent changes</a></li><li><a href="/system:page-tags-list">Page Tags</a></li></ul><ul><li><a href="/admin:manage">Site Manager</a></li></ul><h2><span>Page tags</span></h2>
<style>
@import url(/common--modules/css/Wiki/PagesTagCloud/PagesTagCloudModule.css);

</style>
<p>
It seems you have no tags attached to pages. To attach a tag simply click on the <em>tags</em>
button at the bottom of any page.
</p>

<h2><span>Add a new page</span></h2>
<style>
@import url(/common--modules/css/Wiki/PagesTagCloud/PagesTagCloudModule.css);

</style>
<div class="new-page-box" style="text-align: center; margin: 1em 0">
<form action="dummy.html" method="get" onsubmit="Wikijump.modules.NewPageHelperModule.listeners.create(event)">
<input class="text" name="pageName" type="text" size="15" maxlength="60" style="margin: 1px"><input type="submit" class="button" value="new page" style="margin: 1px">

</form>
</div>

<p style="text-align: center;"><span style="font-size:80%;"><a href="/nav:side">edit this panel</a></span></p>
EOF
        );

        // prettier-ignore
        $nav_top_wikitext_hash = $this->addString(<<<EOF
* [https://wikijump.org Wikijump Blog]
EOF
        );

        // prettier-ignore
        $nav_top_compiled_hash = $this->addString(<<<EOF
<ul>
<li><a href="https://wikijump.com">Wikijump Blog</a></li>
</ul>
EOF
        );

        DB::table('page_category')->insert([
            // Main site (www)

            // ID: 1
            [
                'created_at' => self::TIMESTAMP,
                'site_id' => 1,
                'slug' => '_default',
            ],
            // ID: 2
            [
                'created_at' => self::TIMESTAMP,
                'site_id' => 1,
                'slug' => 'admin',
            ],
            // ID: 3
            [
                'created_at' => self::TIMESTAMP,
                'site_id' => 1,
                'slug' => 'nav',
            ],
            // ID: 4
            [
                'created_at' => self::TIMESTAMP,
                'site_id' => 1,
                'slug' => 'platform',
            ],
            // ID: 5
            [
                'created_at' => self::TIMESTAMP,
                'site_id' => 1,
                'slug' => 'system',
            ],
            // ID: 6
            [
                'created_at' => self::TIMESTAMP,
                'site_id' => 2,
                'slug' => '_default',
            ],

            // Template site (template-en)

            // ID: 7
            [
                'created_at' => self::TIMESTAMP,
                'site_id' => 2,
                'slug' => 'admin',
            ],
            // ID: 8
            [
                'created_at' => self::TIMESTAMP,
                'site_id' => 2,
                'slug' => 'nav',
            ],
            // ID: 9
            [
                'created_at' => self::TIMESTAMP,
                'site_id' => 2,
                'slug' => 'system',
            ],
        ]);

        // Main site (www)
        $this->addPage(
            1,
            1,
            1,
            'start',
            'Welcome to your new Wikijump instance!',
            $www_start_wikitext_hash,
            $www_start_compiled_hash,
        );
        $this->addPage(
            2,
            3,
            1,
            'nav:side',
            'Sidebar',
            $nav_side_wikitext_hash,
            $nav_side_compiled_hash,
        );
        $this->addPage(
            3,
            3,
            1,
            'nav:top',
            'Topbar',
            $nav_top_wikitext_hash,
            $nav_top_compiled_hash,
        );
        $this->addPage(
            4,
            4,
            1,
            'platform:activity',
            'Wikijump: Activity across all sites',
            $platform_activity_wikitext_hash,
            $platform_activity_compiled_hash,
        );
        $this->addPage(
            5,
            4,
            1,
            'platform:sites',
            'Wikijump: List of all sites',
            $platform_sites_wikitext_hash,
            $platform_sites_compiled_hash,
        );
        $this->addPage(
            6,
            4,
            1,
            'platform:search',
            'Wikijump: Search all sites',
            $platform_search_wikitext_hash,
            $platform_search_compiled_hash,
        );
        $this->addPage(
            7,
            5,
            1,
            'system:join',
            'Join this site',
            $system_join_wikitext_hash,
            $system_join_compiled_hash,
        );
        $this->addPage(
            8,
            5,
            1,
            'system:members',
            'Site Members',
            $system_members_wikitext_hash,
            $system_members_compiled_hash,
        );
        $this->addPage(
            9,
            5,
            1,
            'system:recent-changes',
            'Recent Changes',
            $system_recent_changes_wikitext_hash,
            $system_recent_changes_compiled_hash,
        );
        $this->addPage(
            10,
            5,
            1,
            'system:page-tags',
            'Page Tags',
            $system_page_tags_wikitext_hash,
            $system_page_tags_compiled_hash,
        );
        $this->addPage(
            11,
            2,
            1,
            'admin:manage',
            'Site Manager',
            $admin_manage_wikitext_hash,
            $admin_manage_compiled_hash,
        );

        // Template site (template-en)
        $this->addPage(
            12,
            6,
            2,
            'start',
            "You've just created a new Wikijump wiki!",
            $system_join_wikitext_hash,
            $system_join_compiled_hash,
        );
        $this->addPage(
            13,
            8,
            2,
            'nav:side',
            'Sidebar',
            $nav_side_wikitext_hash,
            $nav_side_compiled_hash,
        );
        $this->addPage(
            14,
            8,
            2,
            'nav:top',
            'Topbar',
            $nav_top_wikitext_hash,
            $nav_top_compiled_hash,
        );
        $this->addPage(
            15,
            9,
            2,
            'system:join',
            'Join this site',
            $system_join_wikitext_hash,
            $system_join_compiled_hash,
        );
        $this->addPage(
            16,
            9,
            2,
            'system:members',
            'Site Members',
            $system_members_wikitext_hash,
            $system_members_compiled_hash,
        );
        $this->addPage(
            17,
            9,
            2,
            'system:recent-changes',
            'Recent Changes',
            $system_recent_changes_wikitext_hash,
            $system_recent_changes_compiled_hash,
        );
        $this->addPage(
            18,
            9,
            2,
            'system:page-tags',
            'Page Tags',
            $system_page_tags_wikitext_hash,
            $system_page_tags_compiled_hash,
        );
        $this->addPage(
            19,
            7,
            2,
            'admin:manage',
            'Site Manager',
            $admin_manage_wikitext_hash,
            $admin_manage_compiled_hash,
        );

        // Add links and connections
        $this->addExternalLinks(
            1, // www start
            [
                'https://github.com/scpwiki/wikijump/pulls' => 1,
                'https://github.com/scpwiki/wikijump' => 1,
                'https://github.com/scpwiki/wikijump/tree/legacy' => 1,
                'https://scuttle.atlassian.net/browse/WJ' => 2,
                'https://scuttle.atlassian.net/wiki/spaces/WD/overview' => 1,
                'https://creativecommons.org/licenses/by-sa/4.0/' => 1,
                'https://www.wikidot.com/' => 1,
                'https://wikijump.org' => 1,
                'https://scpwiki.com/forum/c-3335628/general-information' => 1,
                'https://scpwiki.com/forum/c-3335630/feature-requests' => 1,
            ],
        );
        $this->addExternalLinks(
            12, // template-en start
            [
                'https://creativecommons.org/licenses/by-sa/4.0/' => 1,
                'https://www.wikidot.com/' => 1,
                'https://wikijump.org' => 1,
                'https://scpwiki.com/forum/c-3335628/general-information' => 1,
                'https://scpwiki.com/forum/c-3335630/feature-requests' => 1,
            ],
        );
        $this->addExternalLinks(
            3, // www nav:top
            [
                'https://wikijump.org' => 1,
            ],
        );
        $this->addExternalLinks(
            14, // www nav:top
            [
                'https://wikijump.org' => 1,
            ],
        );

        // NOTE: format is [from_page_id, to_page_id, count]

        $this->addInternalLinks([
            // www start
            [1, 11, 2], // admin:manage
            [1, 2, 1], // nav:side
            [1, 3, 1], // nav:top

            // template start
            [12, 19, 2], // admin:manage
            [12, 13, 1], // nav:side
            [12, 14, 1], // nav:top

            // both system:join
            [7, 11, 1], // admin:manage
            [15, 11, 1],

            // both nav:side
            [2, 1, 1], // start
            [2, 4, 1], // platform:activity
            [2, 5, 1], // platform:sites
            [2, 6, 1], // platform:search
            [2, 7, 1], // system:join
            [2, 8, 1], // system:members
            [2, 9, 1], // system:recent-changes
            [2, 10, 1], // system:page-tags
            [2, 11, 1], // admin:manage
            [2, 2, 1], // self

            [13, 12, 1], // start
            [13, 4, 1], // platform:activity
            [13, 5, 1], // platform:sites
            [13, 6, 1], // platform:search
            [13, 15, 1], // system:join
            [13, 16, 1], // system:members
            [13, 17, 1], // system:recent-changes
            [13, 18, 1], // system:page-tags
            [13, 19, 1], // admin:manage
            [13, 13, 1], // self
        ]);
    }

    private function addString(string $value): string
    {
        // Convert to hex because Eloquent doesn't know how to do binary
        $hash = hash('sha512', $value);

        $entries = DB::select(
            "
            SELECT hash FROM text
            WHERE hash = decode(?, 'hex')
            LIMIT 1
        ",
            [$hash],
        );

        if (empty($entries)) {
            DB::insert("INSERT INTO text (hash, contents) VALUES (decode(?, 'hex'), ?)", [
                $hash,
                $value,
            ]);
        }

        return $hash;
    }

    private function addPage(
        int $page_id,
        int $category_id,
        int $site_id,
        string $slug,
        string $title,
        string $wikitext_hash,
        string $compiled_hash,
        int $revision_number = 0
    ): void {
        // NOTE: Assumes the $page_id passed in is the
        //       next one generated when inserting into page.

        // Add page
        DB::insert(
            "INSERT INTO page (
                created_at,
                site_id,
                page_category_id,
                slug
            ) VALUES (
                ?,
                ?,
                ?,
                ?
            )",
            [self::TIMESTAMP, $site_id, $category_id, $slug],
        );

        // Add revision
        DB::insert(
            "INSERT INTO page_revision (
                created_at,
                revision_number,
                page_id,
                site_id,
                user_id,
                changes,
                wikitext_hash,
                compiled_hash,
                compiled_at,
                compiled_generator,
                comments,
                title,
                slug
            ) VALUES (
                ?,
                ?,
                ?,
                ?,
                ?,
                ?,
                decode(?, 'hex'),
                decode(?, 'hex'),
                ?,
                ?,
                ?,
                ?,
                ?
            )",
            [
                self::TIMESTAMP,
                $revision_number,
                $page_id,
                $site_id,
                2,
                self::ALL_CHANGES,
                $wikitext_hash,
                $compiled_hash,
                self::TIMESTAMP,
                'Text_Wiki (seed)',
                '',
                $title,
                $slug,
            ],
        );
    }

    private function addExternalLinks(int $page_id, array $urls): void
    {
        $rows = [];

        foreach ($urls as $url => $count) {
            // If this happens, the structure of the array is incorrect
            // For instance, they may have forgotten the count value.
            if (!is_string($url) || !is_integer($count)) {
                throw new Error("Invalid external link data: url $url, count: $count");
            }

            $rows[] = [
                'page_id' => $page_id,
                'url' => $url,
                'created_at' => self::TIMESTAMP,
                'count' => $count,
            ];
        }

        DB::table('page_link')->insert($rows);
    }

    private function addInternalLinks(array $connections): void
    {
        $rows = [];

        foreach ($connections as $connection) {
            [$from_page_id, $to_page_id, $count] = $connection;

            $rows[] = [
                'from_page_id' => $from_page_id,
                'to_page_id' => $to_page_id,
                'connection_type' => 'link',
                'created_at' => self::TIMESTAMP,
                'count' => $count,
            ];
        }

        DB::table('page_connection')->insert($rows);
    }
}
