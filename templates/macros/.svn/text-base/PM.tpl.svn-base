{**
* @param message
* @param from boolean
* @param to boolean
*}
{defmacro name="pm"}
	<div class="pmessage">
		<div class="header">
			<table>
				{if $from && $message->getFromUserId()}
					<tr>
						<td>
							{t}From{/t}:
						</td>
						<td>
							{printuser user=$message->getFromUser() image=true}
						</td>
					</tr>
				{/if}
				{if $to && $message->getToUserId()!=null}
					<tr>
						<td>
							{t}To{/t}:
						</td>
						<td>
							{printuser user=$message->getToUser() image=true}
						</td>
					</tr>
				{/if}
				<tr>
					<td>
						{t}Subject{/t}:
					</td>
					<td class="subject">
						{$message->getSubject()|escape}
					</td>
				</tr>
				{if $message->getDate()}
					<tr>
						<td>
							{t}Date sent{/t}:
						</td>
						<td>
							<span class="odate">{$message->getDate()->getTimestamp()}|%e %b %Y, %H:%M %Z|agohover</span>
						</td>
					</tr>
				{/if}
			</table>
		</div>
	
		<div class="body">
			{$message->getBody()}
		</div>
	<div>
{/defmacro}