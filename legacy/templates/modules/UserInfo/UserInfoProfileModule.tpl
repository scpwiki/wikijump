<div class="profile-box">

	<h1><img  src="{$avatarUri}" alt="" style="padding: 5px;vertical-align: -50%;"/>{$user->username|escape}</h1>


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

	{t}User since{/t}:  <span class="odate">{$user->created_at->timestamp}|%e %b %Y, %H:%M %Z (%O {t}ago{/t})</span><br/>
	{t}Karma level{/t}:
	{if $user->karma_level == 0}{t}none{/t}
		{elseif $user->karma_level == 1}{t}low{/t}
		{elseif $user->karma_level == 2}{t}medium{/t}
		{elseif $user->karma_level == 3}{t}high{/t}
		{elseif $user->karma_level == 4}{t}very high{/t}
		{elseif $user->karma_level == 5}{t}guru{/t}
	{/if}
	<img src="/user--karma/{$user->id}"/>

</div>

{if $user->bio}
	<div id="ui-profile-included">
		{$user->bio}
	</div>
{/if}
