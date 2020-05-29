<?php /* Smarty version 2.6.7, created on 2009-01-04 19:48:17
         compiled from /var/www/wikidot/templates/modules/managesite/ManageSiteLicenseModule.tpl */ ?>
<?php require_once(SMARTY_CORE_DIR . 'core.load_plugins.php');
smarty_core_load_plugins(array('plugins' => array(array('block', 't', '/var/www/wikidot/templates/modules/managesite/ManageSiteLicenseModule.tpl', 1, false),array('block', 'ltext', '/var/www/wikidot/templates/modules/managesite/ManageSiteLicenseModule.tpl', 84, false),array('modifier', 'escape', '/var/www/wikidot/templates/modules/managesite/ManageSiteLicenseModule.tpl', 28, false),array('modifier', 'replace', '/var/www/wikidot/templates/modules/managesite/ManageSiteLicenseModule.tpl', 85, false),)), $this); ?>
<h1><?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>License<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?></h1>

<p>
	It is <u>very</u> important to clarify the copyright and ownership issues for your site.
	We highly recomment setting an open "<a href="http://en.wikipedia.org/wiki/Copyleft"
	target="_blank">copyleft</a>" license that allows making the
	Content more or less free to copy, modify and use. 
</p>
<p>
	This is particularly important when your Site is created and edited collaboratively.
</p>
<p>
	Read more about <a href="http://creativecommons.org/about/licenses/meet-the-licenses"
	target="_blank">Creative Commons licenses</a>, use a <a href="http://creativecommons.org/license/" 
	target="_blank">wizard</a> to select the proper license or just visit 
	<a href="http://creativecommons.org/" target="_blank">Creative Commons</a>.
</p>

<div>
	<table class="form">
		<tr>
			<td>
				<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Choose the category<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>:
			</td>
			<td>
				<select name="category" size="15" id="sm-license-cats">
					<?php if (count($_from = (array)$this->_tpl_vars['categories'])):
    foreach ($_from as $this->_tpl_vars['category']):
?>
						<option value="<?php echo $this->_tpl_vars['category']->getCategoryId(); ?>
" style="padding: 0 1em" <?php if ($this->_tpl_vars['category']->getName() == '_default'): ?>selected="selected"<?php endif; ?>><?php echo ((is_array($_tmp=$this->_tpl_vars['category']->getName())) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>
</option>
					<?php endforeach; endif; unset($_from); ?>
				</select>
			</td>
			<td>
				<div id="sm-license-noind">
					<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Inherit from <tt>_default</tt><?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>: <input class="checkbox" type="checkbox" id="sm-license-noin"/>
				</div>
			</td>
		</tr>
	</table>
	<div id="sm-license-list">
		<table class="form">
			<tr>
				<td>
					<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Choose the license<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>:
				</td>
				<td>
				
					<select id="sm-license-lic">
						<?php if (count($_from = (array)$this->_tpl_vars['licenses'])):
    foreach ($_from as $this->_tpl_vars['license']):
?>
							<option value="<?php echo $this->_tpl_vars['license']->getLicenseId(); ?>
"><?php echo ((is_array($_tmp=$this->_tpl_vars['license']->getName())) ? $this->_run_mod_handler('escape', true, $_tmp) : smarty_modifier_escape($_tmp)); ?>
</option>
						<?php endforeach; endif; unset($_from); ?>
					</select>
				</td>
			</tr>
		</table>
		<div id="sm-other-license" style="margin: 1em 0">
			<table class="form">
				<tr>
					<td>
						<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>Custom license text<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>:
					</td>
					<td>
						<textarea id="sm-other-license-text" rows="4" cols="50"></textarea>
						<div class="sub">
							(<span id="sm-other-license-text-left"></span> <?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>characters left<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>)
							<br/>
							<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>A few HTML tags are allowed:<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?> &lt;a&gt;, &lt;img/&gt;, &lt;br/&gt;, &lt;strong&gt;, &lt;em&gt;
						</div>
					</td>
				</tr>
			</table>
		</div>	
	</div>
	<div class="buttons">
		<input type="button" value="<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>cancel<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>" id="sm-license-cancel"/>
		<input type="button" value="<?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>save changes<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>" id="sm-license-save"/>
	</div>
	
</div>

<div id="sm-license-preview" style="margin-bottom:2em">
	<h2><?php $this->_tag_stack[] = array('t', array()); smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>License preview<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_t($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>:</h2>
	<?php if (count($_from = (array)$this->_tpl_vars['licenses'])):
    foreach ($_from as $this->_tpl_vars['license']):
?>
		<div id="sm-prev-license-<?php echo $this->_tpl_vars['license']->getLicenseId(); ?>
" class="license-area">
			<?php $this->_tag_stack[] = array('ltext', array('lang' => 'en')); smarty_block_ltext($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>
				<?php echo ((is_array($_tmp=$this->_tpl_vars['license']->getDescription())) ? $this->_run_mod_handler('replace', true, $_tmp, '%%UNLESS%%', 'Unless stated otherwise Content of this page is licensed under') : smarty_modifier_replace($_tmp, '%%UNLESS%%', 'Unless stated otherwise Content of this page is licensed under')); ?>

			<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_ltext($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
			<?php $this->_tag_stack[] = array('ltext', array('lang' => 'pl')); smarty_block_ltext($this->_tag_stack[count($this->_tag_stack)-1][1], null, $this, $_block_repeat=true);while ($_block_repeat) { ob_start(); ?>
				<?php echo ((is_array($_tmp=$this->_tpl_vars['license']->getDescription())) ? $this->_run_mod_handler('replace', true, $_tmp, '%%UNLESS%%', 'Jeśli nie zaznaczono inaczej, Zawartość tej strony dostępna jest na licencji') : smarty_modifier_replace($_tmp, '%%UNLESS%%', 'Jeśli nie zaznaczono inaczej, Zawartość tej strony dostępna jest na licencji')); ?>

			<?php $_block_content = ob_get_contents(); ob_end_clean(); echo smarty_block_ltext($this->_tag_stack[count($this->_tag_stack)-1][1], $_block_content, $this, $_block_repeat=false); }  array_pop($this->_tag_stack); ?>
		</div>
	<?php endforeach; endif; unset($_from); ?>
</div>