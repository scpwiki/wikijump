<h1><a href="javascript:;" onclick="WIKIDOT.modules.AccountModule.utils.loadModule('am-profile')">{t}My profile{/t}</a> / {t}About myself{/t}</h1>

<p>
	{t}The information below is optional but might help.{/t}
</p>	
<p>
	 {t}Note: Each item entered here will be visible to the public. 
	 Please do not enter any information you do not want to disclose.{/t}
</p>

<div>
	<form id="about-form">
		<table class="form">
			<tr>
				<td>{t}Real name{/t}:</td>
				<td><input class="text" name="real_name" type="text" size="40" maxlength="80" value="{$profile->getRealName()|escape}"/></td>
			</tr>
			<tr>
				<td>Gender:</td>
				<td>
					<input class="radio" name="gender" type="radio" value="m" {if $profile->getGender() == "m"}checked="checked"{/if}/> {t}male{/t}<br/>
					<input class="radio" name="gender" type="radio" value="f" {if $profile->getGender() == "f"}checked="checked"{/if}/> {t}female{/t}<br/>
					<input class="radio" name="gender" type="radio" value="" {if $profile->getGender() == ""}checked="checked"{/if}/> {t}do not tell{/t}
				</td>
			</tr>
			
			<tr>
				<td>{t}Birthday{/t}:</td>
				<td>
					<select name="birthday_day">
						<option value="0">{t}Day{/t}</option>
						<option value="1" {if $profile->getBirthdayDay() == 1}selected="selected"{/if}>1st</option>
						<option value="2" {if $profile->getBirthdayDay() == 2}selected="selected"{/if}>2nd</option>
						<option value="3" {if $profile->getBirthdayDay() == 3}selected="selected"{/if}>3rd</option>
						
						<option value="4" {if $profile->getBirthdayDay() == 4}selected="selected"{/if}>4th</option>
						<option value="5" {if $profile->getBirthdayDay() == 5}selected="selected"{/if}>5th</option>
						<option value="6" {if $profile->getBirthdayDay() == 6}selected="selected"{/if}>6th</option>
						<option value="7" {if $profile->getBirthdayDay() == 7}selected="selected"{/if}>7th</option>
						<option value="8" {if $profile->getBirthdayDay() == 8}selected="selected"{/if}>8th</option>
						<option value="9" {if $profile->getBirthdayDay() == 9}selected="selected"{/if}>9th</option>
						<option value="10" {if $profile->getBirthdayDay() == 10}selected="selected"{/if}>10th</option>
						<option value="11" {if $profile->getBirthdayDay() == 11}selected="selected"{/if}>11th</option>
						<option value="12" {if $profile->getBirthdayDay() == 12}selected="selected"{/if}>12th</option>
						
						<option value="13" {if $profile->getBirthdayDay() == 13}selected="selected"{/if}>13th</option>
						<option value="14" {if $profile->getBirthdayDay() == 14}selected="selected"{/if}>14th</option>
						<option value="15" {if $profile->getBirthdayDay() == 15}selected="selected"{/if}>15th</option>
						<option value="16" {if $profile->getBirthdayDay() == 16}selected="selected"{/if}>16th</option>
						<option value="17" {if $profile->getBirthdayDay() == 17}selected="selected"{/if}>17th</option>
						<option value="18" {if $profile->getBirthdayDay() == 18}selected="selected"{/if}>18th</option>
						<option value="19" {if $profile->getBirthdayDay() == 19}selected="selected"{/if}>19th</option>
						<option value="20" {if $profile->getBirthdayDay() == 20}selected="selected"{/if}>20th</option>
						<option value="21" {if $profile->getBirthdayDay() == 21}selected="selected"{/if}>21st</option>
						
						<option value="22" {if $profile->getBirthdayDay() == 22}selected="selected"{/if}>22nd</option>
						<option value="23" {if $profile->getBirthdayDay() == 23}selected="selected"{/if}>23rd</option>
						<option value="24" {if $profile->getBirthdayDay() == 24}selected="selected"{/if}>24th</option>
						<option value="25" {if $profile->getBirthdayDay() == 25}selected="selected"{/if}>25th</option>
						<option value="26" {if $profile->getBirthdayDay() == 26}selected="selected"{/if}>26th</option>
						<option value="27" {if $profile->getBirthdayDay() == 27}selected="selected"{/if}>27th</option>
						<option value="28" {if $profile->getBirthdayDay() == 28}selected="selected"{/if}>28th</option>
						<option value="29" {if $profile->getBirthdayDay() == 29}selected="selected"{/if}>29th</option>
						<option value="30" {if $profile->getBirthdayDay() == 30}selected="selected"{/if}>30th</option>
						
						<option value="31" {if $profile->getBirthdayDay() == 31}selected="selected"{/if}>31st</option>
					</select>
					
					<select name="birthday_month"><option value="0">{t}Month{/t}</option>
						<option value="1" {if $profile->getBirthdayMonth() == 1}selected="selected"{/if}>{t}Jan{/t}</option>
						<option value="2" {if $profile->getBirthdayMonth() == 2}selected="selected"{/if}>{t}Feb{/t}</option>
						<option value="3" {if $profile->getBirthdayMonth() == 3}selected="selected"{/if}>{t}Mar{/t}</option>
						<option value="4" {if $profile->getBirthdayMonth() == 4}selected="selected"{/if}>{t}Apr{/t}</option>
						<option value="5" {if $profile->getBirthdayMonth() == 5}selected="selected"{/if}>{t}May{/t}</option>
						<option value="6" {if $profile->getBirthdayMonth() == 6}selected="selected"{/if}>{t}Jun{/t}</option>
						
						<option value="7" {if $profile->getBirthdayMonth() == 7}selected="selected"{/if}>{t}Jul{/t}</option>
						<option value="8" {if $profile->getBirthdayMonth() == 8}selected="selected"{/if}>{t}Aug{/t}</option>
						<option value="9" {if $profile->getBirthdayMonth() == 9}selected="selected"{/if}>{t}Sep{/t}</option>
						<option value="10" {if $profile->getBirthdayMonth() == 10}selected="selected"{/if}>{t}Oct{/t}</option>
						<option value="11" {if $profile->getBirthdayMonth() == 11}selected="selected"{/if}>{t}Nov{/t}</option>
						<option value="12" {if $profile->getBirthdayMonth() == 12}selected="selected"{/if}>{t}Dec{/t}</option>
					</select>
					<input class="text" name="birthday_year" type="text" value="{if $profile->getBirthdayYear() != ''}{$profile->getBirthdayYear()}{else}{t}Year{/t}{/if}" size="5" maxlength="4" onfocus="if(this.value=='Year')this.value='';"/>
				</td>
			</tr>
			<tr>
				<td>{t}Shortly about myself{/t}:</td>
				<td>
					<textarea id="about-textarea" name="about" cols="40" rows="5">{$profile->getAbout()|escape}</textarea>
					<div class="sub">
						<span id="chleft">200</span> {t}characters left{/t}<br/>
						{t}This is a short description shown when someone clicks on your name
						everywhere it appears.{/t}
					</div>
				</td>
			</tr>
		</table>
		
		<h2>{t}My online presence{/t}</h2>
		
		<table class="form">
			<tr>
				<td>
					{t}My website{/t}:
				</td>
				<td>
					<input  class="text" name="website" type="text" size="40" maxlength="50" value="{$profile->getWebsite()|escape}"/>
					<div class="sub">
						{t}Please start with the <em>http://</em>{/t}
					</div>
				</td>
			</tr>
			<tr>
				<td>{t}Instant messaging{/t}:</td>
				<td>
					<table>
						<tr>
							<td>
								AIM:<br/>
								<input class="text" type="text" name="im_aim" size="20" value="{$profile->getImAim()|escape}"/>
								<br/>
								Gadu-Gadu:<br/>
								<input class="text" type="text" name="im_gadu_gadu" size="20" value="{$profile->getImGaduGadu()|escape}"/>
								<br/>
								Google Talk:<br/>
								<input class="text" type="text" name="im_google_talk" size="20" value="{$profile->getImGoogleTalk()|escape}"/>
								<br/>
								ICQ:<br/>
								<input class="text" type="text" name="im_icq" size="20" value="{$profile->getImIcq()|escape}"/>
							</td>
							<td style="width: 5em">&nbsp;</td>
							<td>
							
								Jabber IM:<br/>
								<input class="text" type="text" name="im_jabber" size="20" value="{$profile->getImJabber()|escape}"/>
								<br/>
								MSN Messenger:<br/>
								<input class="text" type="text" name="im_msn" size="20" value="{$profile->getImMsn()|escape}"/>
								<br/>
								Yahoo! IM:<br/>
								<input class="text" type="text" name="im_yahoo" size="20" value="{$profile->getImYahoo()|escape}"/>
							</td>
						</tr>
					</table>
				</td>
			</tr>
		</table>
	
		<h2>{t}My offline presence{/t}</h2>
		
		<table class="form">
			<tr>
				<td>	{t}Location{/t}:</td>
				<td>
					<input type="text" class="text" name="location" size="40"  value="{$profile->getLocation()|escape}"/>
					<div class="sub">
						{t}city, country + whatever you consider important{/t}
					</div>
				</td>
			</tr>
					
		</table>
		<div class="buttons">
			<input type="button" value="{t}cancel{/t}" onclick="WIKIDOT.modules.AccountModule.utils.loadModule('am-profile')"/>
			<input type="button" value="{t}save{/t}" onclick="WIKIDOT.modules.APAboutModule.listeners.save(event)"/>
		</div>
	</form>
</div>