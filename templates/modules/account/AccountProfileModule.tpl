<h1>{t}My profile{/t}</h1>

<div>
	<h3><a href="javascript:;"  onclick="OZONE.ajax.requestModule('account/profile/APAboutModule', null, WIKIDOT.modules.AccountModule.callbacks.menuClick)">{t}About myself{/t}</a></h3>
	<p>
		{t}Tell others more about yourself by providing additional categorized information such as 
		your website address, location or IM presence.{/t}
	</p>
</div>

<div>
	<h3><a href="http://profiles.{$URL_DOMAIN}/profile:{$user->getUnixName()}">{t}Edit a page about myself{/t}</a></h3>
	<p>
		{t}Edit contents of the page that people see after clicking on your screen name.{/t}
	</p>
</div>

<div>
	<h3><a href="javascript:;"   onclick="OZONE.ajax.requestModule('account/profile/APAvatarModule', null, WIKIDOT.modules.AccountModule.callbacks.menuClick)">{t}My buddy icon (avatar){/t}</a></h3>
	<p>
		{t}Upload or change an icon representing your presence.{/t}
	</p>
</div>

<div>
	<h3><a href="http://{$URL_HOST}/user:info/{$user->getUnixName()}" target="_blank">{t}View my profile{/t}</a></h3>
	<p>
		{t}View your profile as other people see it.{/t}
	</p>
</div>

<div>
	<h3><a href="javascript:;" onclick="WIKIDOT.page.listeners.userInfo({$user->getUserId()})">{t}View my "windowed" profile{/t}</a></h3>
	<p>
		{t}See the window that pops-up when someone clicks on your name.{/t}
	</p>
</div>

<div>
	<h3><a href="javascript:;" onclick="OZONE.ajax.requestModule('account/profile/ChangeScreenNameModule', null, WIKIDOT.modules.AccountModule.callbacks.menuClick)">{t}Change my screen name (NEW!){/t}</a></h3>
	<p>
		{t}Change your screen name, i.e. "{$user->getNickName()|escape}", to a different name.{/t}
	</p>
</div>