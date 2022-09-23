<h1>List of members</h1>

<div>
{foreach from=$memberships item=membership}
	{assign var=user value=$membership->getUser()}
	{printuser user=$user image="yes"}

	<div style="padding-left: 20px">
		member since:  <span class="odate">{$membership->getDateJoined()->getTimestamp()}|%e %b %Y, %H:%M %Z|agohover</span>
		(<a href="javascript:;" onclick="if($('mem-options-{$user->id}').style.display=='none') $('mem-options-{$user->id}').style.display = 'block'; else $('mem-options-{$user->id}').style.display = 'none';">options</a>)

		<div id="mem-options-{$user->id}" style="display: none">

			<a href="javascript:;" onclick="removeUser({$user->id}, '{$user->username}')">remove</a>
			| <a href="javascript:;" onclick="Wikijump.modules.ManageSiteMembersListModule.listeners.removeAndBan({$user->id}, '{$user->username}')">remove &amp; ban</a>
			| <a href="javascript:;" onclick="toModerators({$user->id})">to moderators</a>
			| <a href="javascript:;" onclick="toAdmins({$user->id}, '{$user->username}')">to admins</a>

		</div>
	</div>
{/foreach}
</div>

<div style="display: none" id="remove-user-dialog">
	<h1>Remove user?</h1>
	<p>
		Are you sure you want to remove user <b>%%USER_NAME%%</b> from the members?
	</p>
	<p>
		If the user is also an administrator/moderator of this site their privileges will be lost too.
	</p>
</div>
<div style="display: none" id="remove-ban-user-dialog">
	<h1>Remove and ban user?</h1>
	<p>
		Are you sure you want to remove user <b>%%USER_NAME%%</b> from the members <u>and</u>
		ban from accessing the site in the future?
	</p>
	<p>
		If the user is also an administrator/moderator of this site their privileges will be lost too.
	</p>
</div>
