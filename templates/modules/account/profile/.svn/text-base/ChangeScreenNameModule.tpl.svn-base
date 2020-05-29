<h1><a href="javascript:;" onclick="WIKIDOT.modules.AccountModule.utils.loadModule('am-profile')">{t}Your profile{/t}</a> / {t}Change screen name{/t}</h1>

<p>
	Your current screen name is <strong>{$user->getNickName()|escape}</strong>.
</p>
<p>
	Each Wikidot User can change his/her screen name <strong>twice</strong>. 
	It looks like you have changed yours <strong style="font-size:120%">{$profile->getChangeScreenNameCount()}</strong> time(s).
</p>
<p>
	Although changing the screen name is easy, 
	<strong>we recommed doing so only if you really really need it</strong>.
	If you are sure you want to change it, first you should know a few things:
</p>
<ul>
	<li>
		any such change could confuse people - make sure you are not playing hide and seek
	</li>
	<li>
		at Wikidot we use extensive content caching, so the result of such change will not be visible
		immediately everywhere. At places like forums, your old screen name could be visible for a
		few more days.
	</li>
</ul>
{if $profile->getChangeScreenNameCount() < 2}
	<p>
		O.K., you have been warned. <span style="color: #900">Proceed only if you really need it!</span>
	</p>
	
	<form action="javascript:;" method="get" onsubmit="WIKIDOT.modules.ChangeScreenNameModule.listeners.save(event)">
		<div style="text-align: center">
			Your new screen name: 
			<input type="text" class="text" size="20" id="ap-screen-name-input"/>
			<input type="submit" class="button" value="apply"/>
		</div>
	</form>
	
	<p>
		After you change your screen name you might want to <a style="font-weight: bold; font-size: 120%;" href="javascript:;" onclick="WIKIDOT.page.listeners.logoutClick(event)">log out</a>
		and log in again.
	</p>	
{else}
	<p>
		You have already changed your screen name twice. This option is no longer available to you. So... this option is no longer available to you.
	</p>
{/if}