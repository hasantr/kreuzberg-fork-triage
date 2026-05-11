#
# To learn more about a Podspec see http://guides.cocoapods.org/syntax/podspec.html.
# Run `pod lib lint kreuzberg.podspec` to validate before publishing.
#
Pod::Spec.new do |s|
  s.name             = 'kreuzberg'
  s.version          = '5.0.0-rc.1'
  s.summary          = 'Rust document intelligence library — Flutter FFI plugin for iOS.'
  s.description      = <<-DESC
Flutter FFI plugin wrapping kreuzberg — document text extraction for iOS.
                       DESC
  s.homepage         = 'https://kreuzberg.dev'
  s.license          = { :type => 'Elastic-2.0', :file => '../LICENSE' }
  s.author           = { 'kreuzberg-dev' => 'hello@kreuzberg.dev' }
  s.source           = { :path => '.' }
  s.source_files     = 'Classes/**/*'
  s.ios.deployment_target = '16.0'
  s.vendored_frameworks = 'Frameworks/libkreuzberg_dart.xcframework'
  s.pod_target_xcconfig = { 'DEFINES_MODULE' => 'YES' }
end
