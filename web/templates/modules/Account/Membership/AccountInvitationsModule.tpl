<h1>{t}Invitations{/t}</h1>

<p>
	{t}If someone invites you to join members of a site - you shall receive an invitation which
	you are free to accept or throw away.{/t}
</p>

<h2>{t}Your current invitations{/t}</h2>

{if isset($invitations)}
	{foreach from=$invitations item=invitation}
		{assign var=site value=$invitation->getSite()}
		<table class="form alignleft">
			<tr>
				<td>
					{t}Site{/t}:
				</td>
				<td>
					<a href="{$HTTP_SCHEMA}://{$site->getDomain()}" target="_blank">{$site->getName()|escape}</a>
				</td>
			</tr>
			<tr>
				<td>
					{t}Sent by{/t}:
				</td>
				<td>
					 {printuser user=$invitation->getByUser() image=true}
				</td>
			</tr>
		{if $invitation->getBody()}
			<tr>
				<td>
					{t}Message{/t}:
				</td>
				<td>
					{$invitation->getBody()}
				</td>
			</tr>
		{/if}
		<tr>
			<td>
				{t}Decide{/t}:
			</td>
			<td>
				<a href="javascript:;" onclick="Wikijump.modules.AccountInvitationsModule.listeners.acceptInvitation(event, {$invitation->getInvitationId()})">{t}accept invitation{/t}</a>
				{t}or{/t}
				<a href="javascript:;" onclick="Wikijump.modules.AccountInvitationsModule.listeners.throwAwayInvitation(event, {$invitation->getInvitationId()})">{t}throw away{/t}</a>
			</td>
		</tr>
	</table>
	{/foreach}
{else}
	<p>
		{t}You have no invitations at the moment.{/t}
	</p>
{/if}
