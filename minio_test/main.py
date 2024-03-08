from minio import Minio, Object, error


def _main():
    bucket_name = "se-ci-artifacts"
    minio = Minio(
        "se-ci-storage.localdomain:9000",
        access_key="",
        secret_key="",
        secure=False,
    )
    assert minio.bucket_exists(bucket_name)
    for obj in minio.list_objects(bucket_name):
        print(obj.object_name)


if __name__ == "__main__":
    _main()
