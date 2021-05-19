<h1>Abusive users reports</h1>


{if isset($reps)}
	<table  class="form alignleft">
		<tr>
			<th>
				User
			</th>
			<th>
				Number of flags
			</th>
			<th>
			</th>
		</tr>
		{foreach from=$reps item=rep}
			<tr>
				<td>
					{printuser user=$rep.user image="true"}
				</td>
				<td>
					{$rep.rank}
				</td>
				<td>
					| <a href="javascript:;" onclick="Wikijump.modules.ManageSiteUserAbuseModule.listeners.clear(event, '{$rep.user->id}')">clear flags</a>
					{if $rep.member}
						| <a href="javascript:;" onclick="Wikijump.modules.ManageSiteUserAbuseModule.listeners.removeUser({$rep.user->id}, '{$rep.user->username|escape}')">remove from members</a>
						| <a href="javascript:;" onclick="Wikijump.modules.ManageSiteUserAbuseModule.listeners.removeAndBan({$rep.user->id}, '{$rep.user->username|escape}')">remove from members &amp; ban</a>
					{else}
						| <a href="javascript:;"  onclick="Wikijump.modules.ManageSiteUserAbuseModule.listeners.banUser({$rep.user->id}, '{$rep.user->username|escape}')">ban</a>
					{/if}
				</td>
			</tr>
		{/foreach}
	</table>
{else}
	No abuse reports (yet).
{/if}


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
<div style="display: none" id="ban-user-dialog">
	<h1>Ban user?</h1>
	<p>
		Are you sure you want to ban (block) user <b>%%USER_NAME%%</b>
		from accessing the site in the future?
	</p>
</div>
