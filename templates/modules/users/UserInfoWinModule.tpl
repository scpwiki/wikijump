<div class="owindow" style="width: 60%">
	<div class="title">
		{t}User info{/t}
	</div>
	<div class="content">
		<img style="float:right; padding: 2px 8px; background-color: #FFF;" src="{$avatarUri}" alt="" />
		<h1>{$user->getNickName()|escape}</h1>
		
		{assign var=profile value=$user->getProfile()}
		{if $profile->getRealName()}
		{t}Real name{/t}: {$profile->getRealName()}<br/>
		{/if}
		{if $profile->getGender() == "m" || $profile->getGender() == "f"}
		{t}Gender{/t}: {if $profile->getGender() == "m"}{t}male{/t}{else}{t}female{/t}{/if}<br/>
		{/if}
		{if $profile->getBirthdayDay()}
		{t}Birthday{/t}: {$profile->getBirthdayDate()}<br/>
		{/if}
		{if $profile->getLocation()}
			{t}From{/t}: {$profile->getLocation()|escape}<br/>
		{/if}
		{if $profile->getWebsite()}
			{t}Website{/t}: <a href="{$profile->getWebsite()|escape}">{$profile->getWebsite()|replace:'http://':''|escape}</a><br/>
		{/if}
		
		{t 1=$SERVICE_NAME}User of %1 since{/t}:  <span class="odate">{$user->getRegisteredDate()->getTimestamp()}|%e %b %Y, %H:%M %Z (%O {t}ago{/t})</span><br/>
		
		{if $profile->getAbout()}
			<br/>
			{t}About{/t}: <em>{$profile->getAbout()|escape}</em><br/>
		{/if}
		
		{t}Karma level{/t}: 
		{if $karmaLevel == 0}{t}none{/t}
			{elseif $karmaLevel == 1}{t}low{/t}
			{elseif $karmaLevel == 2}{t}medium{/t}
			{elseif $karmaLevel == 3}{t}high{/t}
			{elseif $karmaLevel == 4}{t}very high{/t}
			{elseif $karmaLevel == 5}{t}guru{/t}
		{/if}
		<img src="/userkarma.php?u={$user->getUserId()}"/>
		{if $member}
			<br/>
			{t}Member of this Site: since{/t} <span class="odate">{$member->getDateJoined()->getTimestamp()}|%e %b %Y, %H:%M %Z (%O {t}ago{/t})</span><br/>
			{if $role}
				{t}Role in this Site{/t}: {if $role=="admin"}{t}Site Administrator{/t}{/if}{if $role=="moderator"}{t}Site Moderator{/t}{/if}
			{/if}
		{/if}
		
		<div style="margin-top: 10px">
		<div style="float:right">
			<a  href="javascript:;" 	onclick="WIKIDOT.modules.UserInfoWinModule.listeners.flagUser(event, {$user->getUserId()})">{t}flag user as abusive{/t}</a>
			<span id="user-abuse-report-button">[?]</span>
		</div>
		<a href="http://{$URL_HOST}/user:info/{$user->getUnixName()}">{t}profile page{/t}</a>
		| <a href="http://{$URL_HOST}/account:you/start/messages/composeto/{$user->getUserId()}">{t}write private message{/t}</a>
		| <a href="javascript:;" onclick="WIKIDOT.modules.UserInfoWinModule.listeners.addContact(event,{$user->getUserId()})">{t}to contacts{/t}</a>
		</div>
	</div>
	<div class="button-bar">
		<a href="javascript:;" onclick="OZONE.dialog.cleanAll()">{t}close window{/t}</a>
	</div>
	
	<div id="user-abuse-report-button-hovertip" style="display: none">
		{t}Notify administrators/moderators about abusive user.{/t}
	</div>
</div>