<div class="mailform-box" id="mailform-box-{$rand}">
	<form id="mailform-{$rand}">
		<table class="form">
			{foreach from=$fields item=field}
				<tr id="mailform-row-{$rand}-{$field.name}">
					<td>
						{if $field.title}
							{$field.title|escape}
						{else}
							{$field.name|escape}
						{/if}
					</td>
					<td>
						<div class="field-error-message"></div>
						{if !$field.type || $field.type == 'text'}
							<input name="{$field.name}" class="text" type="text" 
								{if $field.default}value="{$field.default|escape}"{/if}
								{if $field.size}size="{$field.size}"{else}size="30"{/if}
								{if $field.rules.maxLength}maxlength="{$field.rules.maxLength|escape}"{/if}
							/>
						{/if}
						{if $field.type == "select"}
							<select name="{$field.name}" class="select">
								{if !$field.default}
									<option value="">-- {t}Please select{/t} --</option>
								{/if}
								{foreach from=$field.options key=value item=option}
									<option value="{$value}"
										{if $field.default && $field.default == $value}
											selected="selected"
										{/if}
									>
										{$option|escape}
									</option>
								{/foreach}
							</select>
						{/if}
						{if $field.type == "textarea"}
							<textarea name="{$field.name}" cols="30" rows="5">{$field.default|escape}</textarea>
						{/if}
						{if $field.type == "checkbox"}
							<input name="{$field.name}" type="checkbox" class="checkbox"
								{if $field.default}checked="checked"{/if}/>
						{/if}
						{if $field.hint}
							<div class="sub">
								{$field.hint|escape}
							</div>
						{/if}
					</td>
				</tr>
			{/foreach}
		</table>
		<div class="buttons">
			{*<input type="reset" value="clear form"/>*}
			<input type="button" value="send" onclick="WIKIDOT.modules.MailFormModule.listeners.send(event, '{$rand}')"/>
		</div>
	</form>
	<div id="mailformdef-{$rand}" style="display: none;">
		{$fkey|escape}
	</div>
</div>