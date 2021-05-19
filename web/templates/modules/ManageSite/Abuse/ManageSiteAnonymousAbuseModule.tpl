<h1>Abusive users reports (anonymous users)</h1>


{if isset($reps)}
	<table  class="form alignleft">
		<tr>
			<th>
				IP address
			</th>
			<th>
				WWW Proxy?
			</th>
			<th>
				Number of flags
			</th>
			<th>
			</th>
		</tr>
		{foreach from=$reps item=rep}
			<tr>
				<td>
					{$rep.address}
				</td>
				<td>
					{if $rep.proxy == "t"}yes{else}no{/if}
				</td>
				<td>
					{$rep.rank}
				</td>

				<td>
					| <a href="javascript:;" onclick="Wikijump.modules.ManageSiteAnonymousAbuseModule.listeners.clear(event, '{$rep.address}' {if $rep.proxy == "t"},'proxy'{/if} )">clear flags</a>
					| <a href="javascript:;" onclick="Wikijump.modules.ManageSiteAnonymousAbuseModule.listeners.blockIp(event, '{$rep.address}')" >ban</a>
				</td>
			</tr>
		{/foreach}
	</table>
{else}
	No abuse reports (yet).

{/if}

<div style="display: none" id="ban-ip-dialog">
	<h1>Ban user?</h1>
	<p>
		Are you sure you want to ban (block) address <b>%%IP%%</b>
		from modifying the site as an anonymous user?
	</p>
</div>
