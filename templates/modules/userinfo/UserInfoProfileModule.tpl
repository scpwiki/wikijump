<div class="profile-box">
	
	<h1><img  src="{$avatarUri}" alt="" style="padding: 5px;vertical-align: -50%;"/>{$user->getNickName()|escape}</h1>
	
	
	{assign var=profile value=$user->getProfile()}
	{if $profile->getRealName()}
		{t}Real name{/t}: {$profile->getRealName()}<br/>
	{/if}
	{if $profile->getGender() == "m" || $profile->getGender() == "f"}
		{t}Gender{/t}: {if $profile->getGender() == "m"}male{else}female{/if}<br/>
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
	
	{t}User since{/t}:  <span class="odate">{$user->getRegisteredDate()->getTimestamp()}|%e %b %Y, %H:%M %Z (%O {t}ago{/t})</span><br/>
	{t}Karma level{/t}: 
	{if $karmaLevel == 0}{t}none{/t}
		{elseif $karmaLevel == 1}{t}low{/t}
		{elseif $karmaLevel == 2}{t}medium{/t}
		{elseif $karmaLevel == 3}{t}high{/t}
		{elseif $karmaLevel == 4}{t}very high{/t}
		{elseif $karmaLevel == 5}{t}guru{/t}
	{/if}
	<img src="/userkarma.php?u={$user->getUserId()}"/>
	
</div>

{if $profileContent}
	<div id="ui-profile-included">
		{$profileContent->getText()}
	</div>
{/if}