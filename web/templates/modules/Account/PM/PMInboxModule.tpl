{if $totalPages>1}
	<div style="text-align: center">
		{pager total=$totalPages current=$currentPage jsfunction="inboxPage"}
	</div>
{/if}

<table class="pm-list">
	<tr class="headers">
		<td>&nbsp;</td>

		<td>
			{t}Subject{/t}
		</td>
		<td>
			{t}Sender{/t}
		</td>
		<td>
			{t}Date{/t}
		</td>
		<td>
			&nbsp;
		</td>
	</tr>

	{foreach from=$messages item=message}
		<tr>
			<td>&nbsp;</td>
			<td class="subject">
				<a href="javascript:;"
				{if $message->isUnread()} style="font-weight: bold"{/if}
				onclick="Wikijump.modules.AccountMessagesModule.listeners.viewInboxMessage({$message->id})">{$message->subject|escape}</a>
			</td>
			<td>
				{printuser user=$message->sender image=true}
			</td>
			<td class="date">
				<span class="odate">{$message->created_at->timestamp}|%e %b %Y, %H:%M %Z|agohover</span>
			</td>
			<td>
				<input class="message-select" type="checkbox" id="message-check-{$message->id}"/>
			</td>
		</tr>
	{/foreach}

	<!-- options -->
	{if $messages}
		<tr>
			<td colspan="4"  style="padding: 1em 2em 0 0; text-align: right;">
				<a href="javascript:;" onclick="Wikijump.modules.PMInboxModule.listeners.removeSelected(event)">{t}remove selected{/t}</a>
			</td>
			<td  style="padding-top: 1em">
				[<a href="javascript:;" onclick="Wikijump.modules.PMInboxModule.listeners.selectAll(event)">{t}select all{/t}</a>]
			</td>
		</tr>
	{/if}
</table>


{*
<div style="text-align: right">
{t}with selected{/t}: <a href="javascript:;">{t}delete{/t}</a>
</div>
*}
