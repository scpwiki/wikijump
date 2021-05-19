{**
* Renders the whole row of the form.
* @param string $form name of the form as defined in the xml conf file
* @param string $fieldname name of the field
* @param string $idr - id of the row
* @param string $style
*}
{defmacro name="formrow1"}
	<tr {if isset($idr)} id="{$idr}" {/if} {if $form->isValid($fieldname) == false}class="of-errorrow"{/if} {if isset($style)}style="{$style}"{/if}>
		<td class="of-left">
			{if $form->getExtraAttribute($fieldname, "required") == "true"}
				<span style="color: red; font-family: verdana, Helvetica, sans-serif; font-weight: boldest; vertical-align: -2px;">*</span>
			{/if}
			{$form->getFieldTitle("$fieldname")}
		</td>
		<td class="of-right">
			{macro name="formfield" form=$form fieldname=$fieldname}
		</td>
	</tr>

{/defmacro}

{**
* @param string $form
* @param string $fieldname
*}
{defmacro name="formfield"}
{assign var="fieldType" value=$form->getFieldType($fieldname)}
			{if $fieldType=='text' || $fieldType=='password'}
			<input name="{$form->getFieldLabel($fieldname)}"
				{$form->renderingString($fieldname)}
				{if $form->getFieldType($fieldname) == 'checkbox'}
					{if $form->getFieldValue($fieldname) == true}checked="checked"{/if}
				{else}
					value="{$form->getFieldValue($fieldname)|escape}"
				{/if}
				{if isset($id)} id="{$id}" {/if}
				/>
			{/if}
			{if $fieldType == 'checkbox'}
				<input name="{$form->getFieldLabel($fieldname)}"
				{$form->renderingString($fieldname)}
				{if $form->getFieldValue($fieldname) == true}checked="checked"{/if}
				{if isset($id)} id="{$id}" {/if}
				/>
			{/if}
			{if $fieldType == 'select'}
			<select name="{$form->getFieldLabel($fieldname)}"
				{$form->renderingString($fieldname)}
				{if isset($id)} id="{$id}" {/if}
				>
				{* options follow! *}
				{if $form->getSelectValueListName($fieldname) != null && $form->getSelectValueListName($fieldname) != ''}
					{assign var=listName value=$form->getSelectValueListName($fieldname)}
					{assign var="vals" value=$serviceManager->getService("ListResolver")->getValuesArray($listName) }
				{/if}
				{if $form->getSelectValueTableName($fieldname) != null && $form->getSelectValueTableName($fieldname) != '' }
					{assign var=listName value=$form->getSelectValueTableName($fieldname)}
					{assign var="vals" value=$serviceManager->getService("ListResolver")->getValuesArrayFromTable($listName) }
				{/if}

				{if $form->getFieldValue($fieldname) == '' || $form->getFieldValue($fieldname) == null}
					{assign var="pleaseselectval" value=$serviceManager->getService("ListResolver")->getPleaseSelectValue($listName) }
					{if $pleaseselectval!=null}<option value=" " selected="selected">{$pleaseselectval}</option>{/if}
				{/if}
				{foreach from=$vals key=key item=item}
					<option value="{$key}"
					{if $form->getFieldValue($fieldname) == $key && $form->getFieldValue($fieldname) != null}selected="selected"{/if}
					>{$item}</option>
				{/foreach}

			</select>
			{/if}
			{if $fieldType == 'textarea'}
				<textarea name="{$form->getFieldLabel($fieldname)}"
					{$form->renderingString($fieldname)}
					{if $id!=null} id="{$id}" {/if}
					>{$form->getFieldValue($fieldname)|escape}</textarea>
			{/if}
			{if $fieldType=='file'}
			{* max upload size *}
			{if $form->getUploadMaxSize($fieldname)!= null}
				<input type="hidden" name="MAX_FILE_SIZE"
					value="{$form->getUploadMaxSize($fieldname)}"
				/>
			{/if}
			<input name="{$form->getFieldLabel($fieldname)}"
				{$form->renderingString($fieldname)}
{*				value="{$form->getFieldValue($fieldname)|escape}"*}
			/>
			{/if}
		{*</td>*}
		{*<td>*}
			<!--infobox follows! -->
			{if $form->getHelpText($fieldname) != null}
				&nbsp;
				<span id="{$form->getFieldLabel($fieldname)}-help"
					style="padding-left: 0.2em; padding-right: 0.2em; border: 1px solid #000; font-weight: bold; color: #000; background-color: white; ">?</span>
					<div class="hovertip" id="{$form->getFieldLabel($fieldname)}-help-hovertip">
						{$form->getHelpText($fieldname)}
					</div>
			{/if}

			{* subtitle follows *}
			{if $form->getFieldSubTitle("$fieldname") != ''}
				<br/>
				<span class="fieldsubtitle">
					{$form->getFieldSubTitle("$fieldname")}
				</span>
			{/if}
{/defmacro}

{defmacro name="formrowcustomstart"}
<tr {if isset($idr)} id="{$idr}"{/if}>
{/defmacro}

{defmacro name="formrowcustomend"}
</tr>
{/defmacro}

{**
* Creates a horizontal line (break) within the form
*}
{defmacro name="formtablehr" }
<tr class="separator"><td colspan="2"><hr/></td></tr>
{/defmacro}

{defmacro name="formseparator" }
<tr class="separator"><td colspan="2"><hr/></td></tr>
{/defmacro}

{**
* Creates a horizontal line (break) within the form
*}
{defmacro name="formtablehr2"}
<tr class="separator"><td colspan="2"><hr/></td></tr>
{/defmacro}

{**
* Creates a horizontal line (break) within the form at the top of button field.
*}
{defmacro name="formtablehrbutt"}
<tr>
<td  class="formseparator" colspan="2"><img src="{$ui->image("emptypx.gif")}" height="1" width="1" alt="*"/></td>
</tr>
{/defmacro}

{**
* Creates a title for part of the form.
* @param string $title
* @param string $subtitle (optional)
* @param string $idr
*}
{defmacro name="formparttitle"}
<tr class="formparttitle" {if isset($idr)} id="{$idr}"{/if}>
	<td colspan="2">
		<div class="formparttitle">
		{$title}
		</div>
		{if $subtitle != ''}
		<div class="formpartsubtitle">
			{$subtitle}
		</div>
	{/if}
	</td>
</tr>
{/defmacro}

{**
* Renders <table> element and the top row with border.\
* @param string $title (optional)
* @param string $subtitle (optional)
* @param string $id (optional)
*}
{defmacro name="formtablestart"}
	<div style="text-align: center"> <!-- because IE sucks -->
	<table class="formtable" {if isset($id)}id="{$id}"{/if}>

	{if $title!=''}
		<tr>
			<td colspan="2" style="padding-top: 0.4em">
				<div class="formtitle">
					{$title}
				</div>
				{if $subtitle != ''}
					<div class="formsubtitle">
						{$subtitle}
					</div>
				{/if}
			</td>
		</tr>
<tr class="separator"><td colspan="2"><hr/></td></tr>
{/if}
{/defmacro}

{**
* Renders the bottom border (table row) and the </table> element.
*}
{defmacro name="formtableend"}
</table>
</div>
{/defmacro}

{**
* Renders the whole row of the form with particular field.
*}
{defmacro name="formrowlist1"}
	<tr>
		<td>
			{$form->getFieldTitle("$fieldname")}
		</td>
		<td>
			{assign var="fieldType" value=$form->getFieldType($fieldname)}
			{if $fieldType=='text' || $fieldType=='password'}
				{$form->getFieldValue($fieldname)|escape}
			{/if}
			{if $fieldType == 'select'}
				{assign var="vals" value=$serviceManager->getService("ListResolver")->getValuesArray($form->getSelectValueListName($fieldname)) }
				{assign var="fval" value=$form->getFieldValue($fieldname) }
				{$vals[$fval]}
			{/if}
		</td>
	</tr>

{/defmacro}

{**
* Creates the boottom row with buttons
* @param string $clear_label
* @param string $submit_label
* @param string $submit_event
*}
{defmacro name="formbuttons1"}
	{if $submit_label == null}{assign var="submit_label" value="&nbsp;&nbsp; dalej &nbsp;&nbsp;"}{/if}
	{if $submit_event == null}{assign var="submit_name" value="submit"}
		{else}{assign var="submit_name" value="event_$submit_event"}{/if}

	<tr class="formbuttons">
		<td colspan="2">
			<center>
				{if $clear_label != null}
				<input type="reset" value="{$clear_label}"/>
				{/if}
				<input type="submit" name="{$submit_name}" value="{$submit_label}"/>
			</center>
		</td>
	</tr>

{/defmacro}

{**
* Prints (renders) form errors
* @param Form form
*}
{defmacro name="printFormErrors"}
	{if $form != null}
		{if $form->isValid()==false}
			<div class="of-error">
				<div>
					Please correct the following errors in the form:
					<ul>
						{foreach from=$form->getErrorMessages() item=mess}
							<li>{$mess}</li>
						{/foreach}
					</ul>
				</div>
			</div>
		{/if}
	{/if}
{/defmacro}

{defmacro name="defformhelps"}


{strip}
	{foreach from=$form->getFieldNames() item=fieldName}
		{assign var="helpText" value=$form->getHelpText($fieldName)}
		{if $helpText != null}
			<div id="infobox_tip__{$form->getFieldLabel($fieldName)}" style="position:absolute; visibility:hidden; z-index:200; top:0px; left:0px;">
				<div class="forminfo">
					<div class="forminfohead">
					{$form->getFieldTitle($fieldName)|escape}
					</div>
					<div class="forminfocontent">
						{$helpText|escape}
					</div>
				</div>
			</div>
		{/if}
	{/foreach}
{/strip}

{/defmacro}

{defmacro name="formrow1c"}
	<tr>
		<td class="of-left">
			{$form->getFieldTitle("$fieldname")}
		</td>
		<td class="of-right">
			{assign var="fieldType" value=$form->getFieldType($fieldname)}
			{if $fieldType=='text' || $fieldType=='password'||$fieldType=="textarea"}
				{$form->getFieldValue($fieldname)|escape}
			{/if}
			{if $fieldType == 'select'}
				{assign var="vals" value=$serviceManager->getService("ListResolver")->getValuesArray($form->getSelectValueListName($fieldname)) }
				{assign var="fval" value=$form->getFieldValue($fieldname) }
				{$vals[$fval]}
			{/if}
		</td>
	</tr>

{/defmacro}
