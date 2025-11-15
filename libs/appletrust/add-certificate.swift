import Foundation
import Security

enum SecurityError: Error {
    case generalError
}

func deleteCertificateFromKeyChain(_ certificateLabel: String) -> Bool {
    let delQuery: [NSString: Any] = [
        kSecClass: kSecClassCertificate,
        kSecAttrLabel: certificateLabel,
    ]
    let delStatus: OSStatus = SecItemDelete(delQuery as CFDictionary)

    return delStatus == errSecSuccess
}

func saveCertificateToKeyChain(_ certificate: SecCertificate, certificateLabel: String) throws {
    SecKeychainSetPreferenceDomain(SecPreferencesDomain.system)
    deleteCertificateFromKeyChain(certificateLabel)

    let setQuery: [NSString: AnyObject] = [
        kSecClass: kSecClassCertificate,
        kSecValueRef: certificate,
        kSecAttrLabel: certificateLabel as AnyObject,
        kSecAttrAccessible: kSecAttrAccessibleWhenUnlocked,
    ]
    let addStatus: OSStatus = SecItemAdd(setQuery as CFDictionary, nil)

    guard addStatus == errSecSuccess else {
        throw SecurityError.generalError
    }

    var status = SecTrustSettingsSetTrustSettings(certificate, SecTrustSettingsDomain.admin, nil)
}

func getCertificateFromString(stringData: String) throws -> SecCertificate {
    if let data = NSData(base64Encoded: stringData, options: NSData.Base64DecodingOptions.ignoreUnknownCharacters) {
        if let certificate = SecCertificateCreateWithData(kCFAllocatorDefault, data) {
            return certificate
        }
    }
    throw SecurityError.generalError
}

if CommandLine.arguments.count != 2 {
    print("Usage: \(CommandLine.arguments[0]) [cert.file]")
    print("Usage: \(CommandLine.arguments[0]) --version")
    exit(1)
}

if (CommandLine.arguments[1] == "--version") {
    let version = "dev"
    print(version)
    exit(0)
} else {
    let fileURL = URL(fileURLWithPath: CommandLine.arguments[1])
    do {
        let certData = try Data(contentsOf: fileURL)
        let certificate = SecCertificateCreateWithData(nil, certData as CFData)
        if certificate != nil {
            print("Saving certificate")
            try? saveCertificateToKeyChain(certificate!, certificateLabel: "DropOSS")
            exit(0)
        } else {
            print("ERROR: Unknown error while reading the \(CommandLine.arguments[1]) file.")
        }
    } catch {
        print("ERROR: Unexpected error while reading the \(CommandLine.arguments[1]) file. \(error)")
    }
}
exit(1)