<h3>
	Current signatures:
</h3>
{if $signatures}
	<div style="overflow: auto">
		
		<table id="petition-admin-browse-table" class="form grid">
			<tr>
				<th>
					&nbsp;
				</th>
				<th>
					First name
				</th>
				<th>
					Last name
				</th>
				<th>
					Email
				</th>
				{if $campaign->getCollectAddress()}
					<th>
						Address
					</th>
				{/if}
				{if $campaign->getCollectCity()}
					<th>
						City
					</th>	
				{/if}
				{if $campaign->getCollectState()}
					<th>
						State
					</th>	
				{/if}
				{if $campaign->getCollectZip()}
					<th>
						Zip
					</th>	
				{/if}
				{if $campaign->getCollectCountry()}
					<th>
						Country
					</th>	
				{/if}
				<th>
					Date
				</th>	
				{if $campaign->getCollectComments()}
					<th>
						Comments
					</th>	
				{/if}	
			</tr>
		
		
			{foreach from=$signatures item=sig}
				<tr>
					<td>
						<input type="checkbox" class="checkbox"
							id="petition-signature-check-{$sig->getSignatureId()}"
						/>
					</td>
					<td>
						{$sig->getFirstName()|escape}
					</td>
					<td>
						{$sig->getLastName()|escape}
					</td>
					<td>
						{$sig->getEmail()|escape}
					</td>
					{if $campaign->getCollectAddress()}
						<td>
							{$sig->getAddress1()|escape|replace:' ':'&nbsp;'}
							{if $sig->getAddress2() != ""}
								<br/>{$sig->getAddress2()|escape|replace:' ':'&nbsp;'}
							{/if}
						</td>
					{/if}
					{if $campaign->getCollectCity()}
						<td>
							{$sig->getCity()|escape}
						</td>	
					{/if}
					{if $campaign->getCollectState()}
						<td>
							{$sig->getState()|escape}
						</td>	
					{/if}
					{if $campaign->getCollectZip()}
						<td>
							{$sig->getZip()|escape}
						</td>	
					{/if}
					{if $campaign->getCollectCountry()}
						<td>
							{$sig->getCountry()|escape}
						</td>	
					{/if}
					<td>
						<span class="odate">{$sig->getDate()->getTimestamp()}|%e&nbsp;%b&nbsp;%Y,&nbsp;%H:%M&nbsp;%Z|agohover</span>
					</td>
					{if $campaign->getCollectComments()}
						<td>
							{if $sig->getComments()}
								<div style="width: 40em">
									{$sig->getComments()|escape|nl2br}
								</div>
							{/if}
						</td>	
					{/if}	
				</tr>
			{/foreach}
	
		</table>
	</div>
	
	<p>
		<a href="javascript:;" onclick="WIKIDOT.modules.PetitionAdminModule.listeners.selectAllSignatures(event)">select all</a> 
		|
		<a href="javascript:;" onclick="WIKIDOT.modules.PetitionAdminModule.listeners.deselectAllSignatures(event)">select none</a> 
	</p>
	<p>
		With selected: <a href="javascript:;"  onclick="WIKIDOT.modules.PetitionAdminModule.listeners.removeSelectedSignatures(event, {$campaign->getCampaignId()})">remove</a>
	</p>
{/if}