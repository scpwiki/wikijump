<h1>{t}Welcome{/t}, {$user->username|escape}!</h1>

{if isset($tips)}

	<h2>{t}A few tips just for you{/t}:</h2>

	<ul>
		{if $tips.avatar}
			<li>
				{t}You have not uploaded your buddy icon (avatar) yet. So now you are using a default{/t}
				avatar: <div style="text-align:center">
				<img src="/common--images/avatars/default/a48.png" alt=""/>.
				</div>
				{t escape=no}Go to <em>my profile</em>{/t} >> <em><a href="javascript:;" onclick="OZONE.ajax.requestModule('Account/Profile/APAvatarModule', null, Wikijump.modules.AccountModule.callbacks.menuClick)">{t}my buddy icon{/t}</a></em> {t}to upload your very own image{/t}.
			</li>
		{/if}
	</ul>


{/if}

<div style="float:left; width: 44%; padding: 0 2%">
<h2>Shortcuts:</h2>
<ul>
	<li><a href="/new-site">Get a new wiki</a></li>
	<li><a href="{$HTTP_SCHEMA}://{$URL_HOST}">Go to main page</a></li>

</ul>


</div>

{* CUSTOMIZE *}
{* you can customize this page *}
