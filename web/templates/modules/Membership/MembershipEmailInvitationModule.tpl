<div id="membership-email-invitation-box">

	{if !$invitation}
		<p>
			Sorry, the invitation could not be found. It might have been canceled by the sender, aleady
			used by someone (you?) or the URL link that you were supposed to copy
			from the invitation email might be corrupted somehow.
		</p>
	{else}
		<h2><span>{t}Hi{/t}, {if isset($user)}{printuser user=$user}{else}{$invitation->getName()|escape}{/if}!</span></h2>

		<p>
			It seems you got an invitation from our user {printuser user=$sender image=true} to become
			a member of their Wiki Website <b>{$site->getName()|escape}</b> at <a href="{$HTTP_SCHEMA}://{$site->getDomain()}" target="_blank">{$HTTP_SCHEMA}://{$site->getDomain()}</a>.
		</p>
		<p>
			All you have to do is to accept the invitation and we will instantly add you to members of this Site.
		</p>

		{if isset($user)}
			<p style="padding: 1em; font-size: 180%; text-align: center;font-weight: bold;line-spacing: 120%;">
				<a href="javascript:;" onclick="Wikijump.modules.MembershipEmailInvitationModule.listeners.accept(event, '{$hash}')">accept invitation</a>
			</p>
		{else}
			<p>
				Please create an account (or log in) before you can accept the invitation.
			</p>
			<table style="margin: 1em auto">
				<tr>
					<td style="text-align: center; padding: 1em">
						<div style="font-size: 180%; font-weight: bold;">
							<a href="javascript:;" onclick="WIKIREQUEST.createAccountSkipCongrats=true;Wikijump.page.listeners.loginClick(event)"
								>{t}log in{/t}</a>
						</div>
						<p>
							{t 1=$SERVICE_NAME}if you already have an account at %1{/t}
						</p>
					</td>
					<td style="padding: 1em; font-size: 140%">
						{t}or{/t}
					</td>
					<td style="text-align: center; padding: 1em">
						<div style="font-size: 180%; font-weight: bold;">
							<a href="javascript:;"  onclick="WIKIREQUEST.createAccountSkipCongrats=true; Wikijump.page.listeners.createAccount(event)"
								>{t}create a new account{/t}</a>
						</div>
					</td>
				</tr>
			</table>
		{/if}
	{/if}
</div>
