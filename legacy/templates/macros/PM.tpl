{**
* @param message
* @param from boolean
* @param to boolean
*}
{defmacro name="pm"}
	<div class="pmessage">
		<div class="header">
			<table>
				{if $from && $message->sender->id}
					<tr>
						<td>
							{t}From{/t}:
						</td>
						<td>
							{printuser user=$message->sender image=true}
						</td>
					</tr>
				{/if}
				{if $to && $message->recipient->id != null}
					<tr>
						<td>
							{t}To{/t}:
						</td>
						<td>
							{printuser user=$message->recipient image=true}
						</td>
					</tr>
				{/if}
				<tr>
					<td>
						{t}Subject{/t}:
					</td>
					<td class="subject">
						{$message->subject|escape}
					</td>
				</tr>
				{if $message->created_at}
					<tr>
						<td>
							{t}Date sent{/t}:
						</td>
						<td>
							<span class="odate">{$message->created_at->timestamp}|%e %b %Y, %H:%M %Z|agohover</span>
						</td>
					</tr>
				{/if}
			</table>
		</div>

		<div class="body">
			{$message->body}
		</div>
	<div>
{/defmacro}
