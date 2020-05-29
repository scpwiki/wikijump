<a href="javascript:;" onclick="WIKIDOT.modules.PetitionAdminModule.listeners.viewList(event)">back to the campaign list</a>
{if $campaignsCount>1}
	| 
	{t}choose another campaign{/t}:
	<select onchange="WIKIDOT.modules.PetitionAdminModule.listeners.viewCampaignClick(event, this.value)">
		{foreach from=$campaigns item=camp}
			<option value={$camp->getCampaignId()}
				{if $campaign->getCampaignId() == $camp->getCampaignId()}
					selected="selected"
				{/if}
			>{$camp->getName()|escape}</option>
		{/foreach}
	</select>
{/if}

<h1>{$campaign->getName()|escape}</h1>
<div id="petition-admin-view-tabnav">
	<p style="text-align: center" id="petition-admin-tab-overview">
		{t}overview{/t}
		| 
		<a href="javascript:;" onclick="WIKIDOT.modules.PetitionAdminModule.listeners.browseTabClick(event,  {$campaign->getCampaignId()})"
		>{t}browse signatures{/t}</a>
		|
		<a href="javascript:;" onclick="WIKIDOT.modules.PetitionAdminModule.listeners.downloadTabClick(event,  {$campaign->getCampaignId()})"
		>{t}download data{/t}</a>
	</p>
	<p style="text-align: center;display: none" id="petition-admin-tab-browse" >
		<a href="javascript:;" onclick="WIKIDOT.modules.PetitionAdminModule.listeners.viewCampaignClick(event,{$campaign->getCampaignId()})"
		>{t}overview{/t}</a>
		| 
		{t}browse signatures{/t}
		|
		<a href="javascript:;" onclick="WIKIDOT.modules.PetitionAdminModule.listeners.downloadTabClick(event,  {$campaign->getCampaignId()})"
		>{t}download data{/t}</a>
	</p>
	<p style="text-align: center;display: none" id="petition-admin-tab-download" >
		<a href="javascript:;" onclick="WIKIDOT.modules.PetitionAdminModule.listeners.viewCampaignClick(event,{$campaign->getCampaignId()})"
		>{t}overview{/t}</a>
		| 
		<a href="javascript:;" onclick="WIKIDOT.modules.PetitionAdminModule.listeners.browseTabClick(event,  {$campaign->getCampaignId()})"
		>{t}browse signatures{/t}</a>
		|
		{t}download data{/t}
	</p>
</div>

<div id="petition-admin-view-overview">
	<p>
		{t}Status{/t}: 
		{if $campaign->getActive()}
			<strong>active</strong>
			| 
			<a href="javascript:;"
				onclick="WIKIDOT.modules.PetitionAdminModule.listeners.suspendCampaign(event, {$campaign->getCampaignId()})"
			>{t}suspend this campaign{/t}</a>
		{else}
			<strong>{t}suspended{/t}</strong>
			| 
			<a href="javascript:;"
				onclick="WIKIDOT.modules.PetitionAdminModule.listeners.resumeCampaign(event, {$campaign->getCampaignId()})"
			>{t}resume this campaign{/t}</a>
		{/if}
		<br/>
		{t}Signatures{/t}: {$campaign->getNumberSignatures()}
		
	</p>
	<p>
		{t}More actions{/t}: 
		<a href="javascript:;"
				onclick="WIKIDOT.modules.PetitionAdminModule.listeners.deleteCampaign(event, {$campaign->getCampaignId()})"
		>{t}delete campaign{/t}</a>
	</p>
	
	<h2>
		{t}Collect (and display) following fields{/t}:
	</h2>
	<form id="petition-collect-form">
		<table style="margin: 0 auto">
			<tr>
				<th>
					{t}field{/t}
				</th>
				<th>
					{t}collect{/t}?
				</th>
				<th>
					{t}publicly display{/t}?
				</th>
			</tr>
			<tr>
				<td>
					{t}First and last name{/t}:
				</td>
				<td>
					<input type="checkbox" class="checkbox" checked="checked" disabled="disabled" />
				</td>
				<td>
					<input type="checkbox" class="checkbox" checked="checked" disabled="disabled" />
				</td>
			</tr>
			<tr>
				<td>
					{t}Email{/t}:
				</td>
				<td>
					<input type="checkbox" class="checkbox" checked="checked" disabled="disabled" />
				</td>
				<td>
					<input type="checkbox" class="checkbox" disabled="disabled" />
				</td>
			</tr>
			<tr>
				<td>
					{t}Address{/t}:
				</td>
				<td>
					<input type="checkbox" class="checkbox" 
						{if $campaign->getCollectAddress()}checked="checked"{/if}
						name="collectAddress"/>
				</td>
				<td>
					<input type="checkbox" class="checkbox" disabled="disabled"/>
				</td>
			</tr>
			<tr>
				<td>
					{t}City{/t}:
				</td>
				<td>
					<input type="checkbox" class="checkbox" 
						{if $campaign->getCollectCity()}checked="checked"{/if}
						name="collectCity"/>
				</td>
				<td>
					<input type="checkbox" class="checkbox"
						{if $campaign->getShowCity()}checked="checked"{/if}
						name="showCity"/>
				</td>
			</tr>
			<tr>
				<td>
					{t}State/region/province{/t}:
				</td>
				<td>
					<input type="checkbox" class="checkbox" 
						{if $campaign->getCollectState()}checked="checked"{/if}
						name="collectState"/>
				</td>
				<td>
					<input type="checkbox" class="checkbox"
						{if $campaign->getShowState()}checked="checked"{/if}
						name="showState"/>
				</td>
			</tr>
			<tr>
				<td>
					{t}Zip/postal code{/t}:
				</td>
				<td>
					<input type="checkbox" class="checkbox"
						{if $campaign->getCollectZip()}checked="checked"{/if}
						name="collectZip"/>
				</td>
				<td>
					<input type="checkbox" class="checkbox"
						{if $campaign->getShowZip()}checked="checked"{/if}
						name="showZip"/>
				</td>
			</tr>
			<tr>
				<td>
					{t}Country{/t}:
				</td>
				<td>
					<input type="checkbox" class="checkbox" 
						{if $campaign->getCollectCountry()}checked="checked"{/if}
						name="collectCountry"/>
				</td>
				<td>
					<input type="checkbox" class="checkbox"
						{if $campaign->getShowCountry()}checked="checked"{/if}
						name="showCountry"/>
				</td>
			</tr>
			<tr>
				<td>
					{t}Comments{/t}:
				</td>
				<td>
					<input type="checkbox" class="checkbox" 
						{if $campaign->getCollectComments()}checked="checked"{/if}
						name="collectComments"/>
				</td>
				<td>
					<input type="checkbox" class="checkbox"
						{if $campaign->getShowComments()}checked="checked"{/if}
						name="showComments"/>
				</td>
			</tr>
		</table>
		
		<table class="form">
			<tr>
				<td>
					{t}"Thank you" page{/t}:
				</td>
				<td colspan="2">
					<div class="autocomplete-container" style="width: 20em">
						<input type="text" id="petition-land" class="autocomplete-input text" name="thankYouPage" size="35" value="{$campaign->getThankYouPage()}"/>
						<div id="petition-land-list" class="autocomplete-list"></div>
					</div>
					<div class="sub">
						{t}The users will be taken to this page after confirming the
							signature.<br/>
							Leave blank to produce a standard "Thank you" message.{/t}
					</div>
				</td>
			</tr>
		</table>
		<div class="buttons">
			<input type="button" value="save settings"
			onclick="WIKIDOT.modules.PetitionAdminModule.listeners.saveCollectSettings(event, {$campaign->getCampaignId()})" />
		</div>
	</form>
	
	<p>
		{t}The <em>display</em> option refers to the <a href="#petition-display-paragraph">PetitionList</a>
		module described below.{/t}
	</p>
	
	
	<h2>{t}Instructions{/t}:</h2>
	<p>
		{t}To allow people sign this petition insert the following piece of code into one of your pages:{/t}
	</p>
	<div class="code"><pre>
	[[module SignPetition id="{$campaign->getIdentifier()}"]]</pre>
	</div>
	
	
	<p id="petition-display-paragraph">
		{t}To display the list of recent signatures, use the following code:{/t}
	</p>
	<div class="code"><pre>
	[[module PetitionList id="{$campaign->getIdentifier()}" limit="50"]]</pre>
	</div>
	<p>
		{t}You can use any other number of signatures displayed. The default is 50. 
		Setting limit="0" will print all signatures.{/t}
	</p>
</div>

<div id="petition-admin-view-browse" style="display: none">
	<div class="wait-block">{t}loading{/t}...</div>
</div>
<div id="petition-admin-view-download" style="display: none">
	<p>
		{t}The file coinaining all the signatures can be downloaded in the 
		CSV format (Comma Separated Values). This can be 
		easily imported into major spreadsheet applications such as 
		OpenOffice Calc, Gnumeric or Excel.{/t}
	</p>
	<p style="text-align: center">
		<a href="/default--flow/extra__petition__PetitionDataDownload/campaignId/{$campaign->getCampaignId()}/{$campaign->getIdentifier()}.csv">{$campaign->getIdentifier()}.csv</a>
	</p>
</div>