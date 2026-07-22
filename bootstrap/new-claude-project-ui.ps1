# new-claude-project-ui.ps1 - visual, double-clickable front end for
# new-claude-project.ps1. Both front ends call NewProjectCore.psm1 so project
# creation and cleanup cannot drift:
#
#   * gh mode  : create a PRIVATE GitHub repo from the template, clone it,
#                strip template-only files, drop a README stub, commit, push.
#   * fallback : copy the bundled template locally, initialize Git, then print
#                the manual GitHub steps.
#
# Runtime: Windows PowerShell 5.1, WPF, STA. Launch it through
# New-ClaudeProject-UI.cmd (which supplies the mandatory -STA flag), or:
#   powershell -NoProfile -ExecutionPolicy Bypass -STA -File .\new-claude-project-ui.ps1
#
# Threading: repository and file operations are slow, so they run on a background
# runspace. That worker NEVER touches WPF - it only enqueues plain message
# hashtables onto a synchronized queue. A UI-thread DispatcherTimer is the
# sole consumer: it drains the queue and applies every change to WPF elements,
# so the window never freezes and no control is ever touched off the UI thread.

Add-Type -AssemblyName PresentationFramework
Add-Type -AssemblyName PresentationCore
Add-Type -AssemblyName WindowsBase
Add-Type -AssemblyName System.Windows.Forms

# --- resolve the script directory (needed by the local-fallback copy) --------
$script:ScriptDir = $PSScriptRoot
if ([string]::IsNullOrWhiteSpace($script:ScriptDir)) {
    $script:ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
}
$script:CoreModulePath = Join-Path $script:ScriptDir 'NewProjectCore.psm1'
Import-Module -Force -Name $script:CoreModulePath

$DefaultDest = Join-Path $HOME 'code'
$DefaultTmpl = 'ryanportfolio/AI-Firmware'
$NameRegex   = '^[a-zA-Z0-9._-]+$'

# --- cross-thread channel: a synchronized queue of message hashtables --------
$script:sync = [hashtable]::Synchronized(@{})
$script:sync.Queue = [System.Collections.Queue]::Synchronized((New-Object System.Collections.Queue))

$script:detectJob   = $null
$script:workJob     = $null
$script:running     = $false
$script:formValid   = $false
$script:createdPath = $null
$script:createdRepo = $null

# =============================================================================
#  Background work bodies. These run inside a fresh runspace and NEVER touch
#  WPF; they only enqueue message hashtables for the UI-thread drain to apply.
# =============================================================================

# Detects whether GitHub mode is available and reports the reason.
$DetectBody = {
    param($sync, $ModulePath)

    try {
        Import-Module -Force -Name $ModulePath
        $mode = Get-NewProjectMode
        $sync.Queue.Enqueue(@{ kind = 'mode'; mode = $mode.Mode; reason = $mode.Reason })
    }
    catch {
        $sync.Queue.Enqueue(@{
            kind = 'mode'
            mode = 'local'
            reason = ('Mode detection failed: ' + $_.Exception.Message)
        })
    }
}

# Performs the scaffold through the same module used by the console front end.
$WorkBody = {
    param($sync, $Name, $Dest, $Template, $ModulePath)

    $ErrorActionPreference = 'Stop'

    function Emit($text, $tone) {
        if (-not $tone) { $tone = 'out' }
        $sync.Queue.Enqueue(@{ kind = 'log'; text = [string]$text; tone = $tone })
    }

    try {
        Import-Module -Force -Name $ModulePath
        $logger = {
            param($message, $tone)
            Emit $message $tone
        }
        $result = Invoke-NewProject -Name $Name -Dest $Dest -Template $Template -LogAction $logger

        Emit '' 'out'
        if ($result.Mode -eq 'gh') {
            Emit 'DONE. Private repo created and cloned:' 'ok'
            Emit ("  Local:  {0}" -f $result.LocalPath) 'out'
            if ($result.RemoteUrl) { Emit ("  Remote: {0}" -f $result.RemoteUrl) 'out' }
        }
        else {
            Emit ("DONE (local only). Folder ready: {0}" -f $result.LocalPath) 'ok'
            Emit '' 'out'
            Emit 'To put it on GitHub manually:' 'head'
            Emit ("  1. Create a PRIVATE repo named '{0}' at https://github.com/new" -f $Name) 'out'
            Emit '  2. Then run:' 'out'
            Emit ('       cd "{0}"' -f $result.LocalPath) 'out'
            Emit ("       git remote add origin https://github.com/<your-username>/{0}.git" -f $Name) 'out'
            Emit '       git push -u origin main' 'out'
        }
        Emit '' 'out'
        Emit 'Next: open the folder in Claude Code and run /init-project.' 'head'
        Emit 'Codex users: open the folder in Codex and select the init-project skill.' 'out'

        $sync.Queue.Enqueue(@{
            kind = 'done'
            success = $true
            mode = $result.Mode
            localPath = $result.LocalPath
            remoteUrl = $result.RemoteUrl
        })
    }
    catch {
        $sync.Queue.Enqueue(@{ kind = 'done'; success = $false; message = ('Error: ' + $_.Exception.Message) })
    }
}

# =============================================================================
#  XAML - warm off-white surface, near-black ink, one warm coral accent.
#  The docs.cohere.com aesthetic: calm, roomy, hairline borders, soft corners.
# =============================================================================
$xaml = @'
<Window xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
        xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml"
        Title="New Claude Project"
        Width="800" Height="760" MinWidth="660" MinHeight="620"
        WindowStartupLocation="CenterScreen"
        Background="#FBFAF7"
        FontFamily="Segoe UI"
        UseLayoutRounding="True"
        TextOptions.TextFormattingMode="Ideal"
        TextOptions.TextRenderingMode="ClearType">

  <Window.Resources>
    <SolidColorBrush x:Key="Accent"      Color="#E0553B"/>
    <SolidColorBrush x:Key="Ink"         Color="#1C1B1A"/>
    <SolidColorBrush x:Key="Muted"       Color="#8A847C"/>
    <SolidColorBrush x:Key="Border"      Color="#E0DBD2"/>
    <SolidColorBrush x:Key="Success"     Color="#2E7D4F"/>
    <SolidColorBrush x:Key="Danger"      Color="#C0392B"/>
    <SolidColorBrush x:Key="Warn"        Color="#C9A24B"/>

    <!-- Single-line input field -->
    <Style x:Key="Field" TargetType="TextBox">
      <Setter Property="FontSize" Value="14"/>
      <Setter Property="Foreground" Value="#1C1B1A"/>
      <Setter Property="CaretBrush" Value="#E0553B"/>
      <Setter Property="SelectionBrush" Value="#F2C6BC"/>
      <Setter Property="Height" Value="40"/>
      <Setter Property="Padding" Value="12,0"/>
      <Setter Property="Template">
        <Setter.Value>
          <ControlTemplate TargetType="TextBox">
            <Border x:Name="bd" Background="#FFFFFF" BorderBrush="#E0DBD2"
                    BorderThickness="1" CornerRadius="7">
              <ScrollViewer x:Name="PART_ContentHost" Margin="{TemplateBinding Padding}"
                            VerticalAlignment="Center"/>
            </Border>
            <ControlTemplate.Triggers>
              <Trigger Property="IsKeyboardFocused" Value="True">
                <Setter TargetName="bd" Property="BorderBrush" Value="#E0553B"/>
              </Trigger>
              <Trigger Property="IsEnabled" Value="False">
                <Setter TargetName="bd" Property="Background" Value="#F5F2EC"/>
                <Setter Property="Foreground" Value="#B7ABA3"/>
              </Trigger>
            </ControlTemplate.Triggers>
          </ControlTemplate>
        </Setter.Value>
      </Setter>
    </Style>

    <!-- Primary action: warm coral -->
    <Style x:Key="Primary" TargetType="Button">
      <Setter Property="Foreground" Value="#FFFFFF"/>
      <Setter Property="FontSize" Value="14"/>
      <Setter Property="FontWeight" Value="SemiBold"/>
      <Setter Property="Cursor" Value="Hand"/>
      <Setter Property="MinHeight" Value="42"/>
      <Setter Property="SnapsToDevicePixels" Value="True"/>
      <Setter Property="Template">
        <Setter.Value>
          <ControlTemplate TargetType="Button">
            <Border x:Name="bd" CornerRadius="8" Background="#E0553B" Padding="24,0">
              <ContentPresenter HorizontalAlignment="Center" VerticalAlignment="Center"/>
            </Border>
            <ControlTemplate.Triggers>
              <Trigger Property="IsMouseOver" Value="True">
                <Setter TargetName="bd" Property="Background" Value="#C8462F"/>
              </Trigger>
              <Trigger Property="IsPressed" Value="True">
                <Setter TargetName="bd" Property="Background" Value="#B23E29"/>
              </Trigger>
              <Trigger Property="IsEnabled" Value="False">
                <Setter TargetName="bd" Property="Background" Value="#EADFD9"/>
                <Setter Property="Foreground" Value="#B7ABA3"/>
                <Setter Property="Cursor" Value="Arrow"/>
              </Trigger>
            </ControlTemplate.Triggers>
          </ControlTemplate>
        </Setter.Value>
      </Setter>
    </Style>

    <!-- Secondary / ghost button -->
    <Style x:Key="Ghost" TargetType="Button">
      <Setter Property="Foreground" Value="#2A2825"/>
      <Setter Property="FontSize" Value="13"/>
      <Setter Property="FontWeight" Value="SemiBold"/>
      <Setter Property="Cursor" Value="Hand"/>
      <Setter Property="MinHeight" Value="40"/>
      <Setter Property="SnapsToDevicePixels" Value="True"/>
      <Setter Property="Template">
        <Setter.Value>
          <ControlTemplate TargetType="Button">
            <Border x:Name="bd" CornerRadius="8" Background="#FFFFFF"
                    BorderBrush="#E0DBD2" BorderThickness="1" Padding="16,0">
              <ContentPresenter HorizontalAlignment="Center" VerticalAlignment="Center"/>
            </Border>
            <ControlTemplate.Triggers>
              <Trigger Property="IsMouseOver" Value="True">
                <Setter TargetName="bd" Property="Background" Value="#F4F1EC"/>
                <Setter TargetName="bd" Property="BorderBrush" Value="#D8D2C7"/>
              </Trigger>
              <Trigger Property="IsPressed" Value="True">
                <Setter TargetName="bd" Property="Background" Value="#ECE7DE"/>
              </Trigger>
              <Trigger Property="IsEnabled" Value="False">
                <Setter Property="Foreground" Value="#B7ABA3"/>
                <Setter TargetName="bd" Property="BorderBrush" Value="#ECE7DE"/>
                <Setter Property="Cursor" Value="Arrow"/>
              </Trigger>
            </ControlTemplate.Triggers>
          </ControlTemplate>
        </Setter.Value>
      </Setter>
    </Style>

    <Style x:Key="Eyebrow" TargetType="TextBlock">
      <Setter Property="FontSize" Value="11.5"/>
      <Setter Property="FontWeight" Value="SemiBold"/>
      <Setter Property="Foreground" Value="#57524B"/>
      <Setter Property="Margin" Value="2,0,0,6"/>
    </Style>
  </Window.Resources>

  <Grid Margin="30,26,30,24">
    <Grid.RowDefinitions>
      <RowDefinition Height="Auto"/>
      <RowDefinition Height="Auto"/>
      <RowDefinition Height="Auto"/>
      <RowDefinition Height="*"/>
      <RowDefinition Height="Auto"/>
    </Grid.RowDefinitions>

    <!-- Header / wordmark -->
    <StackPanel Grid.Row="0" Orientation="Horizontal" Margin="2,0,0,20">
      <Border Width="42" Height="42" CornerRadius="11" Background="#E0553B" VerticalAlignment="Center">
        <TextBlock Text="c" Foreground="#FFFFFF" FontSize="24" FontWeight="Bold"
                   HorizontalAlignment="Center" VerticalAlignment="Center" Margin="0,-2,0,0"/>
      </Border>
      <StackPanel Margin="14,0,0,0" VerticalAlignment="Center">
        <TextBlock Text="New Claude Project" Foreground="#1C1B1A" FontSize="21" FontWeight="SemiBold"/>
        <TextBlock Text="Scaffold a private repo from the claude-starter template"
                   Foreground="#8A847C" FontSize="12.5" Margin="0,2,0,0"/>
      </StackPanel>
    </StackPanel>

    <!-- Form card -->
    <Border Grid.Row="1" Background="#FFFFFF" CornerRadius="12"
            BorderBrush="#ECE7DE" BorderThickness="1" Padding="22,20,22,22">
      <StackPanel>

        <TextBlock Text="PROJECT NAME" Style="{StaticResource Eyebrow}"/>
        <TextBox x:Name="TxtName" Style="{StaticResource Field}"/>
        <TextBlock x:Name="TxtNameHint" FontSize="12" Margin="2,7,0,0"
                   Foreground="#8A847C" TextWrapping="Wrap"
                   Text="Letters, digits, dot, dash, underscore. Becomes the folder and repo name."/>

        <TextBlock Text="DESTINATION FOLDER" Style="{StaticResource Eyebrow}" Margin="2,18,0,6"/>
        <Grid>
          <Grid.ColumnDefinitions>
            <ColumnDefinition Width="*"/>
            <ColumnDefinition Width="Auto"/>
          </Grid.ColumnDefinitions>
          <TextBox x:Name="TxtDest" Grid.Column="0" Style="{StaticResource Field}"/>
          <Button x:Name="BtnBrowse" Grid.Column="1" Content="Browse"
                  Style="{StaticResource Ghost}" Margin="10,0,0,0"/>
        </Grid>

        <Expander x:Name="ExpAdvanced" Header="Advanced" Margin="0,16,0,0"
                  Foreground="#57524B" FontSize="12.5">
          <StackPanel Margin="0,12,0,2">
            <TextBlock Text="TEMPLATE REPOSITORY" Style="{StaticResource Eyebrow}"/>
            <TextBox x:Name="TxtTemplate" Style="{StaticResource Field}"/>
            <TextBlock Text="The GitHub template the new repo is created from."
                       FontSize="11.5" Foreground="#8A847C" Margin="2,6,0,0"/>
          </StackPanel>
        </Expander>

      </StackPanel>
    </Border>

    <!-- Mode badge -->
    <Border Grid.Row="2" Margin="0,16,0,0" CornerRadius="9"
            Background="#F3F0EA" BorderBrush="#E6E1D8" BorderThickness="1" Padding="14,10">
      <StackPanel Orientation="Horizontal">
        <Ellipse x:Name="DotMode" Width="9" Height="9" Fill="#C9A24B"
                 VerticalAlignment="Center" Margin="0,0,10,0"/>
        <TextBlock x:Name="TxtModeLabel" Text="Checking..." FontWeight="SemiBold"
                   FontSize="12.5" Foreground="#57524B" VerticalAlignment="Center" Margin="0,0,10,0"/>
        <TextBlock x:Name="TxtModeReason" Text="Looking for the GitHub CLI."
                   FontSize="12.5" Foreground="#8A847C" VerticalAlignment="Center" TextWrapping="Wrap"/>
      </StackPanel>
    </Border>

    <!-- Activity log -->
    <Border Grid.Row="3" Margin="0,18,0,0" CornerRadius="10"
            Background="#F5F2EC" BorderBrush="#E6E1D8" BorderThickness="1">
      <Grid>
        <Grid.RowDefinitions>
          <RowDefinition Height="Auto"/>
          <RowDefinition Height="*"/>
        </Grid.RowDefinitions>
        <Border Grid.Row="0" Background="#EFEAE1" CornerRadius="9,9,0,0"
                BorderBrush="#E6E1D8" BorderThickness="0,0,0,1" Padding="14,9">
          <Grid>
            <StackPanel Orientation="Horizontal" VerticalAlignment="Center">
              <Ellipse Width="10" Height="10" Fill="#E0553B" Margin="0,0,7,0"/>
              <Ellipse Width="10" Height="10" Fill="#C9A24B" Margin="0,0,7,0"/>
              <Ellipse Width="10" Height="10" Fill="#5FA574" Margin="0,0,12,0"/>
              <TextBlock Text="activity" Foreground="#9A938A" FontFamily="Consolas, Courier New"
                         FontSize="12" VerticalAlignment="Center"/>
            </StackPanel>
            <ProgressBar x:Name="PbBusy" Height="3" Width="120" IsIndeterminate="True"
                         Visibility="Collapsed" HorizontalAlignment="Right" VerticalAlignment="Center"
                         Foreground="#E0553B" Background="#E0DBD2" BorderThickness="0"/>
          </Grid>
        </Border>
        <RichTextBox x:Name="LogBox" Grid.Row="1" IsReadOnly="True"
                     IsReadOnlyCaretVisible="False" Background="Transparent"
                     BorderThickness="0" Foreground="#33302B"
                     FontFamily="Consolas, Courier New" FontSize="12.5" Padding="14,12"
                     VerticalScrollBarVisibility="Auto" HorizontalScrollBarVisibility="Auto"/>
      </Grid>
    </Border>

    <!-- Footer -->
    <Grid Grid.Row="4" Margin="0,18,0,0">
      <Grid.ColumnDefinitions>
        <ColumnDefinition Width="*"/>
        <ColumnDefinition Width="Auto"/>
      </Grid.ColumnDefinitions>
      <TextBlock x:Name="TxtStatus" Grid.Column="0" VerticalAlignment="Center"
                 FontSize="12.5" Foreground="#8A847C" TextWrapping="Wrap" Margin="1,0,12,0"/>
      <StackPanel Grid.Column="1" Orientation="Horizontal">
        <Button x:Name="BtnOpenFolder" Content="Open folder" Style="{StaticResource Ghost}"
                Visibility="Collapsed" Margin="0,0,10,0"/>
        <Button x:Name="BtnOpenRepo" Content="Open repo" Style="{StaticResource Ghost}"
                Visibility="Collapsed" Margin="0,0,10,0"/>
        <Button x:Name="BtnCreate" Content="Create project" Style="{StaticResource Primary}"
                IsEnabled="False"/>
      </StackPanel>
    </Grid>
  </Grid>
</Window>
'@

# --- load the window (with a visible fallback if the XAML ever fails) ---------
try {
    [xml]$xmlDoc = $xaml
    $reader = New-Object System.Xml.XmlNodeReader $xmlDoc
    $window = [Windows.Markup.XamlReader]::Load($reader)
}
catch {
    [System.Windows.MessageBox]::Show(
        "Failed to build the window:`r`n" + $_.Exception.Message,
        "New Claude Project", 'OK', 'Error') | Out-Null
    return
}

# --- grab named elements (script scope so handlers/timer can reach them) ------
$script:TxtName       = $window.FindName('TxtName')
$script:TxtNameHint   = $window.FindName('TxtNameHint')
$script:TxtDest       = $window.FindName('TxtDest')
$script:BtnBrowse     = $window.FindName('BtnBrowse')
$script:TxtTemplate   = $window.FindName('TxtTemplate')
$script:DotMode       = $window.FindName('DotMode')
$script:TxtModeLabel  = $window.FindName('TxtModeLabel')
$script:TxtModeReason = $window.FindName('TxtModeReason')
$script:PbBusy        = $window.FindName('PbBusy')
$script:LogBox        = $window.FindName('LogBox')
$script:TxtStatus     = $window.FindName('TxtStatus')
$script:BtnCreate     = $window.FindName('BtnCreate')
$script:BtnOpenFolder = $window.FindName('BtnOpenFolder')
$script:BtnOpenRepo   = $window.FindName('BtnOpenRepo')

$script:AccentBrush  = $window.Resources['Accent']
$script:MutedBrush   = $window.Resources['Muted']
$script:SuccessBrush = $window.Resources['Success']
$script:DangerBrush  = $window.Resources['Danger']
$script:WarnBrush    = $window.Resources['Warn']

# defaults
$script:TxtDest.Text     = $DefaultDest
$script:TxtTemplate.Text = $DefaultTmpl

# --- frozen brushes for the colour-coded log (built once, on the UI thread) ---
function New-FrozenBrush([string]$hex) {
    $b = New-Object System.Windows.Media.SolidColorBrush(
        [System.Windows.Media.ColorConverter]::ConvertFromString($hex))
    $b.Freeze()
    return $b
}
$script:LogBrush = @{
    cmd  = New-FrozenBrush '#C8462F'  # echoed command
    out  = New-FrozenBrush '#33302B'  # normal output
    dim  = New-FrozenBrush '#8A847C'  # quiet chatter
    ok   = New-FrozenBrush '#2E7D4F'  # success
    err  = New-FrozenBrush '#C0392B'  # error
    head = New-FrozenBrush '#1C1B1A'  # emphasis
}

# --- the log document: one paragraph we append coloured Runs to ---------------
$script:LogDoc  = New-Object System.Windows.Documents.FlowDocument
$script:LogDoc.PagePadding = New-Object System.Windows.Thickness(0)
$script:LogDoc.LineHeight  = 16
$script:LogPara = New-Object System.Windows.Documents.Paragraph
$script:LogDoc.Blocks.Add($script:LogPara)
$script:LogBox.Document = $script:LogDoc

# =============================================================================
#  UI helpers (all run on the UI thread)
# =============================================================================
function Add-LogLine($text, $tone) {
    if (-not $tone) { $tone = 'out' }
    $brush = $script:LogBrush[$tone]
    if (-not $brush) { $brush = $script:LogBrush['out'] }
    $run = New-Object System.Windows.Documents.Run([string]$text)
    $run.Foreground = $brush
    if ($tone -eq 'head') { $run.FontWeight = [System.Windows.FontWeights]::SemiBold }
    $script:LogPara.Inlines.Add($run)
    $script:LogPara.Inlines.Add((New-Object System.Windows.Documents.LineBreak))
    $script:LogBox.ScrollToEnd()
}

function Clear-Log {
    $script:LogPara.Inlines.Clear()
}

function Invoke-Background([scriptblock]$body, [object[]]$arguments) {
    $rs = [runspacefactory]::CreateRunspace()
    $rs.ApartmentState = 'MTA'
    $rs.ThreadOptions  = 'ReuseThread'
    $rs.Open()
    $ps = [powershell]::Create()
    $ps.Runspace = $rs
    # Cast to [string] so the body recompiles in the worker runspace,
    # severing scriptblock affinity to this (UI) runspace.
    [void]$ps.AddScript([string]$body)
    foreach ($a in $arguments) { [void]$ps.AddArgument($a) }
    $handle = $ps.BeginInvoke()
    return [pscustomobject]@{ ps = $ps; rs = $rs; handle = $handle }
}

function Close-Worker($job) {
    if ($null -eq $job) { return }
    # If the pipeline is still running (e.g. the user closed the window mid
    # clone, push, or copy), abort it instead of EndInvoke-ing, which would block
    # the UI thread until the external process finishes and freeze the window.
    if ($job.handle -and -not $job.handle.IsCompleted) {
        try { $job.ps.Stop() } catch {}
    } else {
        try { if ($job.ps -and $job.handle) { $job.ps.EndInvoke($job.handle) } } catch {}
    }
    try { $job.ps.Dispose() } catch {}
    try { $job.rs.Close(); $job.rs.Dispose() } catch {}
}

function Set-Running($on) {
    $script:running = $on
    $script:TxtName.IsEnabled     = -not $on
    $script:TxtDest.IsEnabled     = -not $on
    $script:TxtTemplate.IsEnabled = -not $on
    $script:BtnBrowse.IsEnabled   = -not $on
    if ($on) {
        $script:BtnCreate.Content   = 'Working...'
        $script:BtnCreate.IsEnabled = $false
        $script:PbBusy.Visibility   = [System.Windows.Visibility]::Visible
    } else {
        $script:BtnCreate.Content   = 'Create project'
        $script:BtnCreate.IsEnabled = $script:formValid
        $script:PbBusy.Visibility   = [System.Windows.Visibility]::Collapsed
    }
}

# Name validation + live target-collision check. The shared module owns the
# accepted-name contract so the UI and console cannot drift.
function Update-Validation {
    $name = $script:TxtName.Text
    $dest = $script:TxtDest.Text
    $ok = $true

    if ([string]::IsNullOrWhiteSpace($name)) {
        $script:TxtNameHint.Text = 'Letters, digits, dot, dash, underscore. Becomes the folder and repo name.'
        $script:TxtNameHint.Foreground = $script:MutedBrush
        $ok = $false
    }
    elseif (-not (Test-NewProjectName -Name $name)) {
        $script:TxtNameHint.Text = 'Invalid name - use letters, digits, dot, dash, or underscore; all-dot names are not allowed.'
        $script:TxtNameHint.Foreground = $script:DangerBrush
        $ok = $false
    }
    elseif ([string]::IsNullOrWhiteSpace($dest)) {
        $script:TxtNameHint.Text = 'Choose a destination folder.'
        $script:TxtNameHint.Foreground = $script:MutedBrush
        $ok = $false
    }
    else {
        $target = $null
        try { $target = Join-Path $dest $name } catch { $target = $null }
        if ($null -eq $target) {
            $script:TxtNameHint.Text = 'That destination path is not valid.'
            $script:TxtNameHint.Foreground = $script:DangerBrush
            $ok = $false
        }
        elseif (Test-Path -LiteralPath $target) {
            $script:TxtNameHint.Text = "A folder named '$name' already exists in this destination."
            $script:TxtNameHint.Foreground = $script:DangerBrush
            $ok = $false
        }
        else {
            $script:TxtNameHint.Text = "Creates: $target"
            $script:TxtNameHint.Foreground = $script:SuccessBrush
            $ok = $true
        }
    }

    $script:formValid = $ok
    if (-not $script:running) { $script:BtnCreate.IsEnabled = $ok }
}

# =============================================================================
#  Actions + event wiring
# =============================================================================
function Start-Create {
    if ($script:running) { return }
    Update-Validation
    if (-not $script:formValid) { return }

    $name = $script:TxtName.Text.Trim()
    $dest = $script:TxtDest.Text.Trim()
    $tmpl = $script:TxtTemplate.Text.Trim()
    if ([string]::IsNullOrWhiteSpace($dest)) { $dest = $DefaultDest }
    if ([string]::IsNullOrWhiteSpace($tmpl)) { $tmpl = $DefaultTmpl }

    $script:createdPath = $null
    $script:createdRepo = $null
    Clear-Log
    $script:TxtStatus.Text = ''
    $script:BtnOpenFolder.Visibility = [System.Windows.Visibility]::Collapsed
    $script:BtnOpenRepo.Visibility   = [System.Windows.Visibility]::Collapsed

    Add-LogLine ("Scaffolding '{0}' ..." -f $name) 'head'
    Set-Running $true

    Close-Worker $script:workJob
    $script:workJob = Invoke-Background $WorkBody @($script:sync, $name, $dest, $tmpl, $script:CoreModulePath)
}

$script:TxtName.Add_TextChanged({ Update-Validation })
$script:TxtDest.Add_TextChanged({ Update-Validation })

$script:TxtName.Add_KeyDown({
    param($s, $e)
    if ($e.Key -eq 'Return' -and $script:formValid -and -not $script:running) {
        Start-Create
    }
})

$script:BtnBrowse.Add_Click({
    $dlg = New-Object System.Windows.Forms.FolderBrowserDialog
    $dlg.Description = 'Choose the destination folder for the new project'
    $dlg.ShowNewFolderButton = $true
    if (Test-Path -LiteralPath $script:TxtDest.Text) { $dlg.SelectedPath = $script:TxtDest.Text }
    if ($dlg.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
        $script:TxtDest.Text = $dlg.SelectedPath
    }
    $dlg.Dispose()
})

$script:BtnOpenFolder.Add_Click({
    if ($script:createdPath -and (Test-Path -LiteralPath $script:createdPath)) {
        Start-Process -FilePath 'explorer.exe' -ArgumentList ('"{0}"' -f $script:createdPath)
    }
})

$script:BtnOpenRepo.Add_Click({
    if ($script:createdRepo) { Start-Process $script:createdRepo }
})

$script:BtnCreate.Add_Click({ Start-Create })

# --- Dispatcher pump: the SOLE consumer of the queue, on the UI thread --------
$script:Timer = New-Object System.Windows.Threading.DispatcherTimer
$script:Timer.Interval = [TimeSpan]::FromMilliseconds(80)
$script:Timer.Add_Tick({
    while ($script:sync.Queue.Count -gt 0) {
        $item = $script:sync.Queue.Dequeue()
        switch ($item.kind) {
            'log' { Add-LogLine $item.text $item.tone }
            'mode' {
                if ($item.mode -eq 'gh') {
                    $script:TxtModeLabel.Text = 'gh mode'
                    $script:DotMode.Fill = $script:SuccessBrush
                } else {
                    $script:TxtModeLabel.Text = 'Local mode'
                    $script:DotMode.Fill = $script:WarnBrush
                }
                $script:TxtModeReason.Text = $item.reason
                Close-Worker $script:detectJob
                $script:detectJob = $null
            }
            'done' {
                Set-Running $false
                if ($item.success) {
                    $script:createdPath = $item.localPath
                    $script:DotMode.Fill = $script:SuccessBrush
                    if ($item.mode -eq 'gh') {
                        $script:TxtStatus.Text = 'Done. Private repo created, cloned, and pushed.'
                    } else {
                        $script:TxtStatus.Text = 'Done. Project created locally - see the log for the GitHub steps.'
                    }
                    $script:TxtStatus.Foreground = $script:SuccessBrush
                    if ($item.localPath -and (Test-Path -LiteralPath $item.localPath)) {
                        $script:BtnOpenFolder.Visibility = [System.Windows.Visibility]::Visible
                    }
                    if ($item.mode -eq 'gh' -and $item.remoteUrl) {
                        $script:createdRepo = $item.remoteUrl
                        $script:BtnOpenRepo.Visibility = [System.Windows.Visibility]::Visible
                    }
                } else {
                    $script:DotMode.Fill = $script:DangerBrush
                    $script:TxtStatus.Text = $item.message
                    $script:TxtStatus.Foreground = $script:DangerBrush
                    Add-LogLine $item.message 'err'
                }
                Close-Worker $script:workJob
                $script:workJob = $null
            }
        }
    }
})

$window.Add_Closing({
    try { $script:Timer.Stop() } catch {}
    Close-Worker $script:detectJob
    Close-Worker $script:workJob
})

$window.Add_Loaded({ $script:TxtName.Focus() | Out-Null })

# --- seed the log, start the pump, kick off async mode detection --------------
Add-LogLine 'Ready. Fill in a project name and press Create.' 'dim'
Update-Validation
$script:Timer.Start()
$script:detectJob = Invoke-Background $DetectBody @($script:sync, $script:CoreModulePath)

[void]$window.ShowDialog()
