<div id="sign-petition-box">
	<div class="error-block" id="sign-petition-error-box" style="display:none"></div>
	{if $confirmationMode}
	
		<p>
			{t}Please confirm your signature as detailed below{/t}:
		</p>
		<form>
			<table class="form">
				<tr>
				<td>
					{t}First name{/t}:
				</td>
				<td>
					{$signature->getFirstName()|escape}
				</td>
			</tr>
			<tr>
				<td>
					{t}Last name{/t}:
				</td>
				<td>
					{$signature->getLastName()|escape}
				</td>
			</tr>
			{if $campaign->getCollectAddress()}
				<tr>
					<td>
						{t}Address{/t}:
					</td>
					<td>
						{$signature->getAddress1()|escape}
						{if $signature->getAddress2() != ""}
							<br/>{$signature->getAddress2()|escape}
						{/if}
					</td>
				</tr>
			{/if}
			{if $campaign->getCollectCity()}
				<tr>
					<td>
						{t}City{/t}:
					</td>
					<td>
						{$signature->getCity()|escape}
					</td>
				</tr>
			{/if}
			{if $campaign->getCollectState()}
				<tr>
					<td>
						{t}State/province{/t}:
					</td>
					<td>
						{$signature->getState()|escape}
					</td>
				</tr>
			{/if}
			{if $campaign->getCollectZip()}
				<tr>
					<td>
						{t}Zip/postal code{/t}:
					</td>
					<td>
						{$signature->getZip()|escape}
					</td>
				</tr>
			{/if}
			{if $campaign->getCollectCountry()}
				<tr>
					<td>
						{t}Country{/t}:
					</td>
					<td>
						{$signature->getCountry()|escape}
					</td>
				</tr>
			{/if}
			<tr>
				<td>
					{t}Email{/t}:
				</td>
				<td>
					{$signature->getEmail()|escape}
				</td>
			</tr>
			{if $campaign->getCollectComments()}
				<tr id="sign-petition-row-comments">
					<td>
						{t}Comments{/t}:
					</td>
					<td>
						{$signature->getComments()|escape}
					</td>
				</tr>
			{/if}
			</table>
			<div class="buttons">
				<input type="button" value="{t}cancel{/t}"  onclick="WIKIDOT.modules.SignPetitionModule.listeners.cancelSignature(event, {$campaign->getCampaignId()}, '{$signature->getConfirmationHash()}')"/>
				<input type="button" value="{t}confirm{/t}" onclick="WIKIDOT.modules.SignPetitionModule.listeners.confirmSignature(event, {$campaign->getCampaignId()}, '{$signature->getConfirmationHash()}')"/>
			</div>	
		</form>	
		
		
	
	{else}
	
		<form id="sign-petition-form" onsubmit="YAHOO.util.Event.stopEvent(event)" target="dummy.html" action="POST">
			<input type="hidden" name="campaignId" value="{$campaign->getCampaignId()}"/>
			<table class="form">
				<tr id="sign-petition-row-firstName">
					<td>
						{t}First name{/t}:
					</td>
					<td>
						<div class="field-error-message"></div>
						<input name="firstName" type="text" class="text" value="" size="30"/>
					</td>
				</tr>
				<tr id="sign-petition-row-lastName">
					<td>
						{t}Last name{/t}:
					</td>
					<td>
						<div class="field-error-message"></div>
						<input name="lastName" type="text" class="text" value="" size="30"/>
					</td>
				</tr>
				{if $campaign->getCollectAddress()}
					<tr id="sign-petition-row-address">
						<td>
							{t}Address{/t}:
						</td>
						<td>
							<div class="field-error-message"></div>
							<input name="address1" type="text" class="text" value="" size="30"
							style="margin-bottom: 2px"/>
							<br/>
							<input name="address2" type="text" class="text" value="" size="30"/>
						</td>
					</tr>
				{/if}
				{if $campaign->getCollectCity()}
					<tr id="sign-petition-row-city">
						<td>
							{t}City{/t}:
						</td>
						<td>
							<div class="field-error-message"></div>
							<input name="city" type="text" class="text" value="" size="30"/>
						</td>
					</tr>
				{/if}
				{if $campaign->getCollectState()}
					<tr id="sign-petition-row-state">
						<td>
							{t}State/province{/t}:
						</td>
						<td>
							<div class="field-error-message"></div>
							<input name="state" type="text" class="text" value="" size="30"/>
						</td>
					</tr>
				{/if}
				{if $campaign->getCollectZip()}
					<tr id="sign-petition-row-zip">
						<td>
							{t}Zip/postal code{/t}:
						</td>
						<td>
							<div class="field-error-message"></div>
							<input name="zip" type="text" class="text" value="" size="10"/>
						</td>
					</tr>
				{/if}
				{if $campaign->getCollectCountry()}
					<tr id="sign-petition-row-country">
						<td>
							{t}Country{/t}:
						</td>
						<td>
							<div class="field-error-message"></div>
							
							<select name="country">
								<option value="  " selected="selected">(please select a country)</option>
								{*<option value="--">none</option>*}
								{foreach from=$coutryCodes item=country key=code}
									<option value="{$code}">{$country}</option>
								{/foreach}
							</select>
							
							{*<input name="country" type="text" class="text" value="" size="30"/>*}
						</td>
					</tr>
				{/if}
				<tr id="sign-petition-row-email">
					<td>
						{t}Email{/t}:
					</td>
					<td>
						<div class="field-error-message"></div>
						<input name="email" type="text" class="text" value="" size="30"/>
						<div class="sub">
							{t}You will receive a verification email to this address.{/t}
						</div>
					</td>
				</tr>
				{if $campaign->getCollectComments()}
					<tr id="sign-petition-row-comments">
						<td>
							{t}Comments{/t}:
						</td>
						<td>
							<div class="field-error-message"></div>
							<textarea name="comments" class="textarea" cols="30" rows="4"></textarea>
						</td>
					</tr>
				{/if}
			</table>
			<div class="buttons">
				<input type="button" value="{t}sign the petition{/t}" onclick="WIKIDOT.modules.SignPetitionModule.listeners.sign(event)"/>
			</div>
		</form>
	{/if}
</div>