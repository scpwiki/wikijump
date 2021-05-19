<h1>{t}List of sent email invitations{/t}</h1>

{if isset($invitations)}

	<p>
		<a id="sm-invhist-showadminonly" href="javascript:;"
			onclick="Wikijump.modules.ManageSiteInvitationsHistoryModule.listeners.showAdminOnly(event)"
			{if !$showAll}style="font-weight: bold"{/if}>sent by Admins only</a>
		|
		<a id="sm-invhist-showall"
		onclick="Wikijump.modules.ManageSiteInvitationsHistoryModule.listeners.showAll(event)"
		href="javascript:;" {if isset($showAll)}style="font-weight: bold"{/if}>all</a>
	</p>
	<table class="grid form" style="font-size: 87%" id="invitations-history-table">
		<tr>
			<th>
				{t}Date{/t}
			</th>
			<th>
				{t}Recipient{/t}
			</th>
			<th>
				{t}Attempts{/t}
			</th>
			<th>
				{t}Status{/t}
			</th>
			<th>
				{t}Action{/t}
			</th>
		</tr>
	{foreach from=$invitations item=invitation}
		<tr>
			<td>
				<span class="odate">{$invitation->getDate()->getTimestamp()}|%Y-%m-%d </span>
			</td>
			<td>
				{$invitation->getName()|escape} &lt;{$invitation->getEmail()|escape}&gt;
				{if isset($showAll)}
					<br/>
					{t}invited by{/t}<br/>
					{printuser user=$invitation->getUser()}
				{/if}
			</td>
			<td>
				{$invitation->getAttempts()}
			</td>
			<td>
				{if $invitation->getAccepted()}
					{t}joined{/t}
				{elseif !$invitation->getDelivered()}
					{t}delivery failed{/t}
				{else}
					{t}not joined (yet?){/t}
				{/if}
			</td>
			<td>
				{if $invitation->getAccepted()}
				{elseif !$invitation->getDelivered()}
					<a href="javascript:;"
						onclick="Wikijump.modules.ManageSiteInvitationsHistoryModule.listeners.deleteInvitation(event, {$invitation->getInvitationId()}, '{$invitation->getEmail()|escape}')">{t}delete{/t}</a>
				{else}
					{if $invitation->getAttempts()<3}<a href="javascript:;"
					onclick="Wikijump.modules.ManageSiteInvitationsHistoryModule.listeners.resendInvitation(event, {$invitation->getInvitationId()},'{$invitation->getName()|escape}', '{$invitation->getEmail()|escape}')"
					>{t}resend{/t}</a> | {/if}

					<a href="javascript:;"
						onclick="Wikijump.modules.ManageSiteInvitationsHistoryModule.listeners.deleteInvitation(event, {$invitation->getInvitationId()}, '{$invitation->getEmail()|escape}')">{t}delete{/t}</a>
				{/if}
			</td>
		</tr>
	{/foreach}

	</table>

	<p>
		You can resend (remind) the invitation 2 times max. Please do not abuse this.
	</p>

	<div id="resend-invitations-form" style="display: none">
		<h2>Resend invitation to <span id="resend-invitations-to"></span></h2>

		<p>
			You can write a few words here if you wish:
		</p>
		<form>
			<textarea id="resend-invitations-message" cols="20" rows="5" style="width: 95%">Hi, I just want to remind you about the invitation I have sent you some time ago.</textarea>

			<p>
				Original message (with the invitation link) will be included too.
			</p>
			<div class="buttons">
				<input type="button" value="{t}cancel{/t}" onclick="$('resend-invitations-form').style.display = 'none'"/>
				<input type="button" value="{t}send reminder{/t}" onclick="Wikijump.modules.ManageSiteInvitationsHistoryModule.listeners.resendInvitation2(event)"/>
			</div>
		</form>
	</div>
{/if}

<p style="font-weight: bold; text-align: center;">
	<a href="javascript:;" onclick="Wikijump.modules.ManagerSiteModule.utils.loadModule('sm-email-invitations');">send more invitations</a>
</p>
