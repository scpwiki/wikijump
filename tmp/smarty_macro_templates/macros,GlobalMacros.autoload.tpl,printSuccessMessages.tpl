  
	{if $messages != null}
	<div style="text-align: center">
	<table class="successmess">
		<tr>
			<td>
				<img src="{$ui->image("warning1.png")}" alt="*"/>
			</td>
			<td class="messholder">
		{foreach from=$messages item="message"}
			{$message}
		{/foreach}
			</td>
		</tr>
	</table>
	</div>
	{/if}
 