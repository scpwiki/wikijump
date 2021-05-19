{if $totalPages>1}
	<div style="text-align: center">
		{pager total=$totalPages current	=$currentPage jsfunction="sentPage"}
	</div>
{/if}

<table class="pm-list">
	<tr class="headers">
		<td>&nbsp;</td>

		<td>
			{t}Subject{/t}
		</td>
		<td>
			{t}Recepient{/t}
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
				onclick="Wikijump.modules.AccountMessagesModule.listeners.viewSentMessage({$message->getMessageId()})">{$message->getSubject()|escape}</a>
			</td>
			<td>
				{printuser user=$message->getToUser() image=true}
			</td>
			<td class="date">
				<span class="odate">{$message->getDate()->getTimestamp()}|%e %b %Y, %H:%M %Z|agohover</span>
			</td>
			<td>
				<input class="message-select" type="checkbox" id="message-check-{$message->getMessageId()}"/>
			</td>
		</tr>
	{/foreach}

	<!-- options -->
	{if isset($messages)}
		<tr>
			<td colspan="4"  style="padding: 1em 2em 0 0; text-align: right;">
				<a href="javascript:;" onclick="Wikijump.modules.PMSentModule.listeners.removeSelected(event)">{t}remove selected{/t}</a>
			</td>
			<td  style="padding-top: 1em">
				[<a href="javascript:;" onclick="Wikijump.modules.PMSentModule.listeners.selectAll(event)">{t}select all{/t}</a>]
			</td>
		</tr>
	{/if}
</table>


