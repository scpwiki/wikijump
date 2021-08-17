<h1>{t}My profile{/t}</h1>

<div>
	<h3><a href="javascript:;"  onclick="OZONE.ajax.requestModule('Account/Profile/APAboutModule', null, Wikijump.modules.AccountModule.callbacks.menuClick)">{t}About myself{/t}</a></h3>
	<p>
		{t}Tell others more about yourself by providing additional categorized information such as
		your website address, location or IM presence.{/t}
	</p>
</div>

<div>
	<h3><a href="javascript:;"   onclick="OZONE.ajax.requestModule('Account/Profile/APAvatarModule', null, Wikijump.modules.AccountModule.callbacks.menuClick)">{t}My buddy icon (avatar){/t}</a></h3>
	<p>
		{t}Upload or change an icon representing your presence.{/t}
	</p>
</div>

<div>
	<h3><a href="{$HTTP_SCHEMA}://{$URL_HOST}/user:info/{$user->unix_name}" target="_blank">{t}View my profile{/t}</a></h3>
	<p>
		{t}View your profile as other people see it.{/t}
	</p>
</div>

<div>
	<h3><a href="javascript:;" onclick="Wikijump.page.listeners.userInfo({$user->id})">{t}View my "windowed" profile{/t}</a></h3>
	<p>
		{t}See the window that pops-up when someone clicks on your name.{/t}
	</p>
</div>

<div>
	<h3><a href="javascript:;" onclick="OZONE.ajax.requestModule('Account/Profile/ChangeScreenNameModule', null, Wikijump.modules.AccountModule.callbacks.menuClick)">{t}Change my screen name (NEW!){/t}</a></h3>
	<p>
		{t}Change your screen name, i.e. "{$user->username|escape}", to a different name.{/t}
	</p>
</div>
