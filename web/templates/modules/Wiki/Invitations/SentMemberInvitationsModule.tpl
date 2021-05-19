<div id="sent-member-invitations-box">

	<p style="text-align: right">
		<a href="javascript:;" onclick="Wikijump.modules.SentMemberInvitationsModule.listeners.sendMore(event)">send more invitations</a>
	</p>

	{if isset($invitations)}
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
							onclick="Wikijump.modules.SentMemberInvitationsModule.listeners.deleteInvitation(event, {$invitation->getInvitationId()}, '{$invitation->getEmail()|escape}')">{t}delete{/t}</a>
					{else}
						{if $invitation->getAttempts()<3}<a href="javascript:;"
						onclick="Wikijump.modules.SentMemberInvitationsModule.listeners.resendInvitation(event, {$invitation->getInvitationId()},'{$invitation->getName()|escape}', '{$invitation->getEmail()|escape}')"
						>{t}resend{/t}</a> | {/if}

						<a href="javascript:;"
							onclick="Wikijump.modules.SentMemberInvitationsModule.listeners.deleteInvitation(event, {$invitation->getInvitationId()}, '{$invitation->getEmail()|escape}')">{t}delete{/t}</a>
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
					<input type="button" value="{t}send reminder{/t}" onclick="Wikijump.modules.SentMemberInvitationsModule.listeners.resendInvitation2(event)"/>
				</div>
			</form>
		</div>
	{else}
		You have not sent any invitations to this Wiki (yet).
	{/if}
</div>
