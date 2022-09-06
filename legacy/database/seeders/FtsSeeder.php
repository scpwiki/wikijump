<?php

namespace Database\Seeders;

use Illuminate\Database\Seeder;
use Illuminate\Support\Facades\DB;

// prettier-ignore
class FtsSeeder extends Seeder
{
    /**
     * Run the database seeds.
     *
     * @return void
     */
    public function run()
    {
        DB::unprepared(
            <<<STATEMENT
INSERT INTO public.fts_entry (fts_id, page_id, title, unix_name, thread_id, site_id, text, vector) VALUES (32, 32, 'Top', 'nav:top', NULL, 2, '


example menu

submenu


contact

', '''contact'':7 ''exampl'':4 ''menu'':5 ''nav'':2C ''submenu'':6 ''top'':1C,3C');
INSERT INTO public.fts_entry (fts_id, page_id, title, unix_name, thread_id, site_id, text, vector) VALUES (33, 33, 'Template', 'profile:template', NULL, 2, '

Profile has not been created (yet).
', '''creat'':8 ''profil'':2C,4 ''templat'':1C,3C ''yet'':9');
INSERT INTO public.fts_entry (fts_id, page_id, title, unix_name, thread_id, site_id, text, vector) VALUES (34, 5, 'Side', 'nav:side', NULL, 2, '


Welcome page


What is a Wiki Site?
How to edit pages?


How to join this site?
Site members


Recent changes
List all pages
Page Tags


Site Manager

Page tags


Add a new page


edit this panel
', '''add'':33 ''chang'':23 ''edit'':13,37 ''join'':17 ''list'':24 ''manag'':30 ''member'':21 ''nav'':2C ''new'':35 ''page'':5,14,26,27,31,36 ''panel'':39 ''recent'':22 ''side'':1C,3C ''site'':10,19,20,29 ''tag'':28,32 ''welcom'':4 ''wiki'':9');
INSERT INTO public.fts_entry (fts_id, page_id, title, unix_name, thread_id, site_id, text, vector) VALUES (37, 36, 'Congratulations, welcome to your new wiki!', 'start', NULL, 2, '

If this is your first site
Then there are some things you need to know:

You can configure all security and other settings online, using the Site Manager. When you invite other people to help build this site they don''t have access to the Site Manager unless you make them administrators like yourself. Check out the Permissions section.
Your Wikidot site has two menus, one at the side called ''nav:side'', and one at the top called ''nav:top''. These are Wikidot pages, and you can edit them like any page.
To edit a page, go to the page and click the Edit button at the bottom. You can change everything in the main area of your page. The Wikidot system is easy to learn and powerful.
You can attach images and other files to any page, then display them and link to them in the page.
Every Wikidot page has a history of edits, and you can undo anything. So feel secure, and experiment.
To start a forum on your site, see the Site Manager » Forum.
The license for this Wikidot site has been set to Creative Commons Attribution-Share Alike 3.0 License. If you want to change this, use the Site Manager.
If you want to learn more, make sure you visit the Documentation section at www.wikidot.org

More information about the Wikidot project can be found at www.wikidot.org.
', '''3.0'':203 ''access'':50 ''administr'':59 ''alik'':202 ''anyth'':168 ''area'':123 ''attach'':138 ''attribut'':200 ''attribution-shar'':199 ''bottom'':115 ''build'':43 ''button'':112 ''call'':77,85 ''chang'':118,209 ''check'':62 ''click'':109 ''common'':198 ''configur'':25 ''congratul'':1C ''creativ'':197 ''display'':147 ''document'':226 ''easi'':131 ''edit'':95,101,111,163 ''everi'':156 ''everyth'':119 ''experi'':173 ''feel'':170 ''file'':142 ''first'':12 ''forum'':177,186 ''found'':238 ''go'':104 ''help'':42 ''histori'':161 ''imag'':139 ''inform'':231 ''invit'':38 ''know'':22 ''learn'':133,219 ''licens'':188,204 ''like'':60,97 ''link'':150 ''main'':122 ''make'':57,221 ''manag'':35,54,184,214 ''menus'':72 ''nav'':78,86 ''need'':20 ''new'':5C ''one'':73,81 ''onlin'':31 ''page'':91,99,103,107,126,145,155,158 ''peopl'':40 ''permiss'':65 ''power'':135 ''project'':235 ''section'':66,227 ''secur'':27,171 ''see'':181 ''set'':30,195 ''share'':201 ''side'':76,79 ''site'':13,34,45,53,69,180,183,192,213 ''start'':7C,175 ''sure'':222 ''system'':129 ''thing'':18 ''top'':84,87 ''two'':71 ''undo'':167 ''unless'':55 ''use'':32,211 ''visit'':224 ''want'':207,217 ''welcom'':2C ''wiki'':6C ''wikidot'':68,90,128,157,191,234 ''www.wikidot.org'':229,240 ''�'':185');
INSERT INTO public.fts_entry (fts_id, page_id, title, unix_name, thread_id, site_id, text, vector) VALUES (45, 13, 'How To Edit Pages - Quickstart', 'how-to-edit-pages', NULL, 2, '

If you are allowed to edit pages in this Site, simply click on edit button at the bottom of the page. This will open an editor with a toolbar pallette with options.
To create a link to a new page, use syntax: [[[new page name]]] or [[[new page name | text to display]]]. Follow the link (which should have a different color if page does not exist) and create a new page and edit it!
Although creating and editing pages is easy, there are a lot more options that allows creating powerful sites. Please visit Documentation pages (at wikidot.org) to learn more.
', '''allow'':14,99 ''although'':85 ''bottom'':28 ''button'':25 ''click'':22 ''color'':71 ''creat'':44,78,86,100 ''differ'':70 ''display'':62 ''document'':105 ''easi'':91 ''edit'':3C,9C,16,24,83,88 ''editor'':36 ''exist'':76 ''follow'':63 ''how-to-edit-pag'':6C ''learn'':110 ''link'':46,65 ''lot'':95 ''name'':55,59 ''new'':49,53,57,80 ''open'':34 ''option'':42,97 ''page'':4C,10C,17,31,50,54,58,73,81,89,106 ''pallett'':40 ''pleas'':103 ''power'':101 ''quickstart'':5C ''simpli'':21 ''site'':20,102 ''syntax'':52 ''text'':60 ''toolbar'':39 ''use'':51 ''visit'':104 ''wikidot.org'':108');
INSERT INTO public.fts_entry (fts_id, page_id, title, unix_name, thread_id, site_id, text, vector) VALUES (39, 37, 'List of all wikis', 'system-all:all-sites', NULL, 1, '

Below is the list of public visible Wikis hosted at this service:

', '''all-sit'':8C ''host'':19 ''list'':1C,14 ''public'':16 ''servic'':22 ''site'':10C ''system'':6C ''system-al'':5C ''visibl'':17 ''wiki'':4C,18');
INSERT INTO public.fts_entry (fts_id, page_id, title, unix_name, thread_id, site_id, text, vector) VALUES (40, 38, 'List wikis by tags', 'system-all:sites-by-tags', NULL, 1, '



', '''list'':1C ''site'':9C ''sites-by-tag'':8C ''system'':6C ''system-al'':5C ''tag'':4C,11C ''wiki'':2C');
INSERT INTO public.fts_entry (fts_id, page_id, title, unix_name, thread_id, site_id, text, vector) VALUES (42, 40, 'Activity across all wikis', 'system-all:activity', NULL, 1, '




Recent edits (all wikis)



Top Sites


Top Forums


New users


Some statistics




', '''across'':2C ''activ'':1C,8C ''edit'':10 ''forum'':16 ''new'':17 ''recent'':9 ''site'':14 ''statist'':20 ''system'':6C ''system-al'':5C ''top'':13,15 ''user'':18 ''wiki'':4C,12');
INSERT INTO public.fts_entry (fts_id, page_id, title, unix_name, thread_id, site_id, text, vector) VALUES (53, 1, 'Manage', 'admin:manage', NULL, 1, '


', '''admin'':2C ''manag'':1C,3C');
INSERT INTO public.fts_entry (fts_id, page_id, title, unix_name, thread_id, site_id, text, vector) VALUES (54, 2, 'You', 'account:you', NULL, 1, '


', '''account'':1C');
INSERT INTO public.fts_entry (fts_id, page_id, title, unix_name, thread_id, site_id, text, vector) VALUES (55, 3, 'Get a new wiki', 'new-site', NULL, 1, '

Use this simple form to create a new wiki.
To admins: you can customize this page by simply clicking "edit" at the bottom of the page.

', '''admin'':18 ''bottom'':30 ''click'':26 ''creat'':13 ''custom'':21 ''edit'':27 ''form'':11 ''get'':1C ''new'':3C,6C,15 ''new-sit'':5C ''page'':23,33 ''simpl'':10 ''simpli'':25 ''site'':7C ''use'':8 ''wiki'':4C,16');
INSERT INTO public.fts_entry (fts_id, page_id, title, unix_name, thread_id, site_id, text, vector) VALUES (56, 4, 'Info', 'user:info', NULL, 1, '


', '''info'':1C,3C ''user'':2C');
INSERT INTO public.fts_entry (fts_id, page_id, title, unix_name, thread_id, site_id, text, vector) VALUES (57, 24, 'Search All Wikis', 'search:all', NULL, 1, '


', '''search'':1C,4C ''wiki'':3C');
INSERT INTO public.fts_entry (fts_id, page_id, title, unix_name, thread_id, site_id, text, vector) VALUES (58, 25, 'Search This Wiki', 'search:site', NULL, 1, '


', '''search'':1C,4C ''site'':5C ''wiki'':3C');
INSERT INTO public.fts_entry (fts_id, page_id, title, unix_name, thread_id, site_id, text, vector) VALUES (59, 26, 'Search Users', 'search:users', NULL, 1, '

To look for someone, please enter:

email address of a person you are looking for (this will look for exact match)
any part of the screen name or realname (lists all Users matching the query)


', '''address'':12 ''email'':11 ''enter'':10 ''exact'':24 ''list'':34 ''look'':6,18,22 ''match'':25,37 ''name'':31 ''part'':27 ''person'':15 ''pleas'':9 ''queri'':39 ''realnam'':33 ''screen'':30 ''search'':1C,3C ''someon'':8 ''user'':2C,4C,36');
INSERT INTO public.fts_entry (fts_id, page_id, title, unix_name, thread_id, site_id, text, vector) VALUES (38, 22, 'Side', 'nav:side', NULL, 1, '


Welcome page


What is a Wiki?
How to edit pages?
Get a new wiki!

All wikis

Recent activity
All wikis
Wikis by tags
Search

This wiki

How to join this site?
Site members


Recent changes
List all pages
Page Tags


Site Manager

Page tags


Add a new page


edit this panel
', '''activ'':21 ''add'':48 ''chang'':38 ''edit'':12,52 ''get'':14 ''join'':32 ''list'':39 ''manag'':45 ''member'':36 ''nav'':2C ''new'':16,50 ''page'':5,13,41,42,46,51 ''panel'':54 ''recent'':20,37 ''search'':27 ''side'':1C,3C ''site'':34,35,44 ''tag'':26,43,47 ''welcom'':4 ''wiki'':9,17,19,23,24,29');
INSERT INTO public.fts_entry (fts_id, page_id, title, unix_name, thread_id, site_id, text, vector) VALUES (41, 39, 'Search', 'system-all:search', NULL, 1, '


Search all Wikis
Perform a search through all public and visible wikis.



Search users
To look for someone, please enter:

email address of a person you are looking for (this will look for exact match)
any part of the screen name or realname (lists all Users matching the query)



', '''address'':27 ''email'':26 ''enter'':25 ''exact'':39 ''list'':49 ''look'':21,33,37 ''match'':40,52 ''name'':46 ''part'':42 ''perform'':9 ''person'':30 ''pleas'':24 ''public'':14 ''queri'':54 ''realnam'':48 ''screen'':45 ''search'':1C,5C,6,11,18 ''someon'':23 ''system'':3C ''system-al'':2C ''user'':19,51 ''visibl'':16 ''wiki'':8,17');
INSERT INTO public.fts_entry (fts_id, page_id, title, unix_name, thread_id, site_id, text, vector) VALUES (47, 43, 'Wiki Members', 'system:members', NULL, 1, '

Members:


Moderators


Admins

', '''admin'':7 ''member'':2C,4C,5 ''moder'':6 ''system'':3C ''wiki'':1C');
INSERT INTO public.fts_entry (fts_id, page_id, title, unix_name, thread_id, site_id, text, vector) VALUES (49, 45, 'Recent changes', 'system:recent-changes', NULL, 1, '


', '''chang'':2C,6C ''recent'':1C,5C ''recent-chang'':4C ''system'':3C');
INSERT INTO public.fts_entry (fts_id, page_id, title, unix_name, thread_id, site_id, text, vector) VALUES (52, 48, 'Page Tags', 'system:page-tags', NULL, 1, '




', '''page'':1C,5C ''page-tag'':4C ''system'':3C ''tag'':2C,6C');
INSERT INTO public.fts_entry (fts_id, page_id, title, unix_name, thread_id, site_id, text, vector) VALUES (43, 23, 'Welcome to your new Wikidot installation!', 'start', NULL, 1, '

Congratulations, you have successfully installed Wikidot software on your computer!
What to do next
Customize this wiki
Wikidot consists of several wiki sites, not just one. Right now you are on the main wiki. Customize it!

You can configure all security and other settings online, using the Site Manager. When you invite other people to help build this site they don''t have access to the Site Manager unless you make them administrators like yourself. Check out the Permissions section.
Your Wikidot site has two menus, one at the side called ''nav:side'', and one at the top called ''nav:top''. These are Wikidot pages, and you can edit them like any page.
To edit a page, go to the page and click the Edit button at the bottom. You can change everything in the main area of your page. The Wikidot system is easy to learn and powerful.
You can attach images and other files to any page, then display them and link to them in the page.
Every Wikidot page has a history of edits, and you can undo anything. So feel secure, and experiment.
To start a forum on your site, see the Site Manager » Forum.
The license for this Wikidot site has been set to Creative Commons Attribution-Share Alike 3.0 License. If you want to change this, use the Site Manager.
If you want to learn more, make sure you visit the Documentation section at www.wikidot.org

Customize default template
Default initial template for other wikis is located at template-en. If someone creates a new wiki, this one is cloned to the new address. A good thing to do is to go to template-en and customize it.
Create more templates
Simply create wikis with unix names starting with "template-" (e.g. "template-pl", "template-blog") and your users will be able to choose which wiki they want to start with.
Visit Wikidot.org
Go to www.wikidot.org — home of the Wikidot software — for extra documentation, howtos, tips and support.

More information about the Wikidot project can be found at www.wikidot.org.
Search all wikis


Search users

', '''3.0'':223 ''abl'':318 ''access'':71 ''address'':278 ''administr'':80 ''alik'':222 ''anyth'':189 ''area'':144 ''attach'':159 ''attribut'':220 ''attribution-shar'':219 ''blog'':312 ''bottom'':136 ''build'':64 ''button'':133 ''call'':98,106 ''chang'':139,229 ''check'':83 ''choos'':320 ''click'':130 ''clone'':274 ''common'':218 ''comput'':17 ''configur'':46 ''congratul'':8 ''consist'':26 ''creat'':267,294,298 ''creativ'':217 ''custom'':22,42,250,292 ''default'':251,253 ''display'':168 ''document'':246,340 ''e.g'':306 ''easi'':152 ''edit'':116,122,132,184 ''en'':264,290 ''everi'':177 ''everyth'':140 ''experi'':194 ''extra'':339 ''feel'':191 ''file'':163 ''forum'':198,206 ''found'':353 ''go'':125,286,330 ''good'':280 ''help'':63 ''histori'':182 ''home'':333 ''howto'':341 ''imag'':160 ''inform'':346 ''initi'':254 ''instal'':6C,12 ''invit'':59 ''learn'':154,239 ''licens'':208,224 ''like'':81,118 ''link'':171 ''locat'':260 ''main'':40,143 ''make'':78,241 ''manag'':56,75,205,234 ''menus'':93 ''name'':302 ''nav'':99,107 ''new'':4C,269,277 ''next'':21 ''one'':33,94,102,272 ''onlin'':52 ''page'':112,120,124,128,147,166,176,179 ''peopl'':61 ''permiss'':86 ''pl'':309 ''power'':156 ''project'':350 ''right'':34 ''search'':356,359 ''section'':87,247 ''secur'':48,192 ''see'':202 ''set'':51,215 ''sever'':28 ''share'':221 ''side'':97,100 ''simpli'':297 ''site'':30,55,66,74,90,201,204,212,233 ''softwar'':14,337 ''someon'':266 ''start'':7C,196,303,326 ''success'':11 ''support'':344 ''sure'':242 ''system'':150 ''templat'':252,255,263,289,296,305,308,311 ''template-blog'':310 ''template-en'':262,288 ''template-pl'':307 ''thing'':281 ''tip'':342 ''top'':105,108 ''two'':92 ''undo'':188 ''unix'':301 ''unless'':76 ''use'':53,231 ''user'':315,360 ''visit'':244,328 ''want'':227,237,324 ''welcom'':1C ''wiki'':24,29,41,258,270,299,322,358 ''wikidot'':5C,13,25,89,111,149,178,211,336,349 ''wikidot.org'':329 ''www.wikidot.org'':249,332,355');
INSERT INTO public.fts_entry (fts_id, page_id, title, unix_name, thread_id, site_id, text, vector) VALUES (44, 41, 'What Is A Wiki', 'what-is-a-wiki', NULL, 1, '

According to Wikipedia, the world largest wiki site:

A Wiki ([ˈwiː.kiː] &lt;wee-kee&gt; or [ˈwɪ.kiː] &lt;wick-ey&gt;) is a type of website that allows users to add, remove, or otherwise edit and change most content very quickly and easily.

And that is it! As a part of a farm of wikis this site is a great tool that you can use to publish content, upload files, communicate and collaborate.
', '''accord'':10 ''add'':40 ''allow'':37 ''chang'':46 ''collabor'':82 ''communic'':80 ''content'':48,77 ''easili'':52 ''edit'':44 ''ey'':30 ''farm'':62 ''file'':79 ''great'':69 ''kee'':24 ''kiː'':21,27 ''largest'':15 ''otherwis'':43 ''part'':59 ''publish'':76 ''quick'':50 ''remov'':41 ''site'':17,66 ''tool'':70 ''type'':33 ''upload'':78 ''use'':74 ''user'':38 ''websit'':35 ''wee'':23 ''wee-ke'':22 ''what-is-a-wiki'':5C ''wick'':29 ''wick-ey'':28 ''wiki'':4C,9C,16,19,64 ''wikipedia'':12 ''world'':14 ''ˈwiː'':20 ''ˈwɪ'':26');
INSERT INTO public.fts_entry (fts_id, page_id, title, unix_name, thread_id, site_id, text, vector) VALUES (48, 44, 'How to join this wiki?', 'system:join', NULL, 1, '


Please change this page according to your policy (configure first using Site Manager) and remove this note.

Who can join?
You can write here who can become a member of this site.
Join!
So you want to become a member of this site? Tell us why and apply now!


Or, if you already know a "secret password", go for it!

', '''accord'':12 ''alreadi'':60 ''appli'':55 ''becom'':34,45 ''chang'':9 ''configur'':16 ''first'':17 ''go'':65 ''join'':3C,7C,27,40 ''know'':61 ''manag'':20 ''member'':36,47 ''note'':24 ''page'':11 ''password'':64 ''pleas'':8 ''polici'':15 ''remov'':22 ''secret'':63 ''site'':19,39,50 ''system'':6C ''tell'':51 ''us'':52 ''use'':18 ''want'':43 ''wiki'':5C ''write'':30');
INSERT INTO public.fts_entry (fts_id, page_id, title, unix_name, thread_id, site_id, text, vector) VALUES (46, 42, 'How To Edit Pages', 'how-to-edit-pages', NULL, 1, '

If you are allowed to edit pages in this Site, simply click on edit button at the bottom of the page. This will open an editor with a toolbar pallette with options.
To create a link to a new page, use syntax: [[[new page name]]] or [[[new page name | text to display]]]. Follow the link (which should have a different color if page does not exist) and create a new page and edit it!
Although creating and editing pages is easy, there are a lot more options that allows creating powerful sites. Please visit Documentation pages (at wikidot.org) to learn more.
', '''allow'':13,98 ''although'':84 ''bottom'':27 ''button'':24 ''click'':21 ''color'':70 ''creat'':43,77,85,99 ''differ'':69 ''display'':61 ''document'':104 ''easi'':90 ''edit'':3C,8C,15,23,82,87 ''editor'':35 ''exist'':75 ''follow'':62 ''how-to-edit-pag'':5C ''learn'':109 ''link'':45,64 ''lot'':94 ''name'':54,58 ''new'':48,52,56,79 ''open'':33 ''option'':41,96 ''page'':4C,9C,16,30,49,53,57,72,80,88,105 ''pallett'':39 ''pleas'':102 ''power'':100 ''simpli'':20 ''site'':19,101 ''syntax'':51 ''text'':59 ''toolbar'':38 ''use'':50 ''visit'':103 ''wikidot.org'':107');
INSERT INTO public.fts_entry (fts_id, page_id, title, unix_name, thread_id, site_id, text, vector) VALUES (50, 46, 'List all pages', 'system:list-all-pages', NULL, 1, '


', '''list'':1C,6C ''list-all-pag'':5C ''page'':3C,8C ''system'':4C');
INSERT INTO public.fts_entry (fts_id, page_id, title, unix_name, thread_id, site_id, text, vector) VALUES (51, 47, 'Page Tags List', 'system:page-tags-list', NULL, 1, '


', '''list'':3C,8C ''page'':1C,6C ''page-tags-list'':5C ''system'':4C ''tag'':2C,7C');
INSERT INTO public.fts_entry (fts_id, page_id, title, unix_name, thread_id, site_id, text, vector) VALUES (60, 49, 'Log in', 'auth:login', NULL, 1, '


', '''auth'':2C ''log'':1C ''login'':3C');
INSERT INTO public.fts_entry (fts_id, page_id, title, unix_name, thread_id, site_id, text, vector) VALUES (61, 50, 'Create account - step 1', 'auth:newaccount', NULL, 1, '


', '''1'':4C ''account'':2C ''auth'':5C ''creat'':1C ''newaccount'':6C ''step'':3C');
INSERT INTO public.fts_entry (fts_id, page_id, title, unix_name, thread_id, site_id, text, vector) VALUES (62, 51, 'Create account - step 2', 'auth:newaccount2', NULL, 1, '


', '''2'':4C ''account'':2C ''auth'':5C ''creat'':1C ''newaccount2'':6C ''step'':3C');
INSERT INTO public.fts_entry (fts_id, page_id, title, unix_name, thread_id, site_id, text, vector) VALUES (63, 52, 'Create account - step 3', 'auth:newaccount3', NULL, 1, '


', '''3'':4C ''account'':2C ''auth'':5C ''creat'':1C ''newaccount3'':6C ''step'':3C');
STATEMENT

        );
    }
}
