
<div>
	{if isset($memberships)}
	<table>
		{foreach from=$memberships item=membership}
			{assign var=user value=$membership->getUser()}
			<tr>
				<td>{printuser user=$user image="yes"}</td>
				{if $from!="admins" && $from != "moderators" && $showSince}<td style="padding-left: 2em">{t}since{/t} <span class="odate">{$membership->getDateJoined()->getTimestamp()}|%e %b %Y, %H:%M %Z (%O {t}ago{/t})</span></td>{/if}
			</tr>
		{/foreach}
	</table>
	{else}
	{t}No users.{/t}
	{/if}
</div>
