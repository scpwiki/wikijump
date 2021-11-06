<div class="owindow" style="width: 60%">
	<div class="title">
		{t}User info{/t}
	</div>
	<div class="content">
		<img style="float:right; padding: 2px 8px; background-color: #FFF;" src="{$avatarUri}" alt="" />
		<h1>{$user->username|escape}</h1>

        {if $user->real_name}
            {t}Real name{/t}: {$user->real_name}<br/>
        {/if}
        {if $user->pronouns}
            {t}Pronouns{/t}: {$user->pronouns}<br/>
        {/if}
        {if $user->dob}
            {t}Birthday{/t}: {$user->dob}<br/>
        {/if}
        {if $user->about_page}
            {t}About page{/t}: <a href="{$user->about_page|escape}">{$user->about_page|replace:'https?://':''|escape}</a><br/>
        {/if}

		{t 1=$SERVICE_NAME}User of %1 since{/t}:  <span class="odate">{$user->created_at->timestamp}|%e %b %Y, %H:%M %Z (%O {t}ago{/t})</span><br/>

		{if $user->bio}
			<br/>
			{t}About{/t}: <em>{$user->bio|escape}</em><br/>
		{/if}

		{t}Karma level{/t}:
        {if $user->karma_level == 0}{t}none{/t}
            {elseif $user->karma_level == 1}{t}low{/t}
            {elseif $user->karma_level == 2}{t}medium{/t}
            {elseif $user->karma_level == 3}{t}high{/t}
            {elseif $user->karma_level == 4}{t}very high{/t}
            {elseif $user->karma_level == 5}{t}guru{/t}
        {/if}
		<img src="/user--karma/{$user->id}"/>
		{if $member}
			<br/>
			{t}Member of this Site: since{/t} <span class="odate">{$member->getDateJoined()->getTimestamp()}|%e %b %Y, %H:%M %Z (%O {t}ago{/t})</span><br/>
			{if $role}
				{t}Role in this Site{/t}: {if $role=="admin"}{t}Site Administrator{/t}{/if}{if $role=="moderator"}{t}Site Moderator{/t}{/if}
			{/if}
		{/if}

		<div style="margin-top: 10px">
		<a href="{$HTTP_SCHEMA}://{$URL_HOST}/user:info/{$user->slug}">{t}profile page{/t}</a>
		| <a href="{$HTTP_SCHEMA}://{$URL_HOST}/account:you/start/messages/composeto/{$user->id}">{t}write private message{/t}</a>
		| <a href="javascript:;" onclick="Wikijump.modules.UserInfoWinModule.listeners.addContact(event,{$user->id})">{t}to contacts{/t}</a>
		</div>
	</div>
	<div class="button-bar">
		<a href="javascript:;" onclick="OZONE.dialog.cleanAll()">{t}close window{/t}</a>
	</div>
</div>
