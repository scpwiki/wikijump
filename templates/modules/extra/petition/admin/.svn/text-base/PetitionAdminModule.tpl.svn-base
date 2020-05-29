{if !$withoutBox}<div class="petition-admin-module-box" id="petition-admin-module-box">{/if}
<h1>{t}Administer your petition campaigns{/t}</h1>
	
	{if $campaigns}
		{* print petitions *}
		
		<table class="campaign-list form grid">
			<tr>
				<th>
					{t}Name{/t}
				</th>
				<th>
					{t}Signatures{/t}
				</th>
				<th>
					{t}Status{/t}
				</th>
			</tr>
			{foreach from=$campaigns item=campaign}
				<tr>
					<td>
						<a href="javascript:;"
							onclick="WIKIDOT.modules.PetitionAdminModule.listeners.viewCampaignClick(event, {$campaign->getCampaignId()})"
						>{$campaign->getName()|escape}</a>
					</td>
					<td>
						{$campaign->getNumberSignatures()|escape}
					</td>
					<td>
						{if $campaign->getActive()}{t}active{/t}{else}{t}suspended{/t}{/if}
					</td>
				</tr>
			{/foreach}
		</table>
	
	{else}
		<p>
			{t}You have no petition campaigns defined.{/t}
		</p>
	{/if}
	<p>
		<a href="javascript:;" onclick="WIKIDOT.modules.PetitionAdminModule.listeners.newCampaignClick(event)">+ {t}create a new campaign{/t}</a>
	</p>
	
	<div id="petition-new-campain-box" style="display: none">
		<h2>{t}Create a new camaign{/t}</h2>
		
		<form id="petition-new-campain-form">
			<div class="error-block" id="petition-new-campaign-error-box" style="display: none"></div>
			<table class="form">
				<tr>
					<td>
						{t}Name{/t}:
					</td>
					<td>
						<input type="text" class="text" name="name" size="30" value=""/>
					</td>
				</tr>
				<tr>
					<td>
						{t}Identifier{/t}:
					</td>
					<td>
						<input type="text" class="text" name="identifier" size="30" value=""/>
						<div class="sub">
							{t}Only letters and numbers allowed, e.g. <em>p01</em>.<br/>
							This will be used to point other modules <br/>
							to the particular campaign with the <em>id="..."</em> attribute.{/t}
						</div>
					</td>
				</tr>
			</table>
			
			<div class="buttons">
				<input type="button" value="{t}cancel{/t}" onclick="WIKIDOT.modules.PetitionAdminModule.listeners.cancelNewCampaignClick(event)"/>
				<input type="button" value="{t}create{/t}" onclick="WIKIDOT.modules.PetitionAdminModule.listeners.createCampaign(event)" />
				
			</div>
		</form>
	
	</div>
{if !$withoutBox}</div>{/if}