
	{if $messages != null}
	<div style="text-align: center">
	<table class="errormess">
		<tr>
			<td>
				<img src="{$ui->image("warningsymbol.png")}" alt="*"/>
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

